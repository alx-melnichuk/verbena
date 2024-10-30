import {
  ChangeDetectionStrategy, Component, ElementRef, HostListener, Input, OnChanges, Renderer2, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';

const LABEL_DAYS_DEF = 'days';
const ATTR_IS_ACTIVE = 'is-active';
const CSS_BOX_JUSTIFY_CONTENT = '--dtt-box-justify-content';
const CSS_LEADING_ZERO = '--dtt-leading-zero';
const CSS_LETTER_SPACING = '--dtt-letter-spacing';
const CSS_INCREMENT = '--dtt-increment';
const CCS_MAX_SECONDS = '--dtt-max-seconds'
const CSS_DAYS = '--dtt-days';
const CSS_HOURS = '--dtt-hours';
const CSS_MINUTES = '--dtt-minutes';
const CSS_SECONDS = '--dtt-seconds';

export type LabelDaysType = { [key: number]: string };

/*
  <!-- Displays the current time. ("HH : MM : SS") -->
  <app-date-time-timer [isActive]="true"></app-date-time-timer>

  <!-- If "startDate" is greater than the current date, a timer is displayed that counts down to the specified "startDate".
      (" 1 days  HH : MM : SS") -->
  <app-date-time-timer [isActive]="true" [startDate]="startDate"></app-date-time-timer>

  <!-- If "startDate" is less than the current date, a timer is displayed counting down from the specified "startDate".
      (" 1 days  HH : MM : SS") -->
  <app-date-time-timer [isActive]="true" [startDate]="startDate"></app-date-time-timer>

  <!-- The "labelDays" parameter, as a string, specifies the name of the label for the number of days.
  <app-date-time-timer [isActive]="true" [labelDays]="'days'" [startDate]="startDate"></app-date-time-timer> -->

  <!-- The "labelDays" parameter, as an object, defines the name of the label for a specific number of days.
       Value 0 - corresponds to the default label name. -->
  <app-date-time-timer [isActive]="true" [startDate]="startDate"
    [labelDays]="{ '0': 'days', '1': 'only days', '2': 'total days' }">
  </app-date-time-timer>

  <!-- The number of days is displayed if the value is greater than zero. -->
*/
@Component({
  selector: 'app-date-time-timer',
  exportAs: 'appDateTimeTimer',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './date-time-timer.component.html',
  styleUrls: ['./date-time-timer.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class DateTimeTimerComponent implements OnChanges {
  @Input()
  public startDate: Date | null | undefined;
  @Input()
  public isActive: boolean | null | undefined;
  @Input()
  public isAlignCenter: boolean | null | undefined; // together with "isHideLeadingZero"
  @Input()
  public isHideDays: boolean | null | undefined;
  @Input()
  public isHideLeadingZero: boolean | null | undefined;  // together with "isAlignCenter"
  @Input()
  public labelDays: string | LabelDaysType | null | undefined;
  @Input()
  public letterSpacing: number | null | undefined = 0; // in 'px'

  public innLabelDays: string = '';
  public labelDaysObj: LabelDaysType = this.getLabelDaysObj(null);

  public days: number = -1;
  public hours: number = 0;
  public minutes: number = 0;
  public seconds: number = 0;
  
  private isEvenIteration: boolean = false;
  private settimeoutId: number | null = null;
  private isDocumentVisible: boolean = true;

  constructor(
    private renderer: Renderer2,
    public hostRef: ElementRef<HTMLElement>,
  ) {
  }

  @HostListener('document:visibilitychange', ['$event'])
  listenerVisibilityChange() {
    this.isDocumentVisible = !document.hidden;
    this.modifyCurrValueAndSetTimeout();
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['labelDays']) {
      this.labelDaysObj = this.getLabelDaysObj(this.labelDays)
    }
    if (!!changes['isActive']) {
      if (this.isActive) {
        this.modifyCurrValueAndSetTimeout();
      } else {
        this.clearTimeout();
        this.clearCurrValue();
      }
      HtmlElemUtil.setAttr(this.renderer, this.hostRef, ATTR_IS_ACTIVE, !!this.isActive ? '' : null);
    }
    if (!!changes['isAlignCenter']) {
      HtmlElemUtil.setProperty(this.hostRef, CSS_BOX_JUSTIFY_CONTENT, !!this.isAlignCenter ? 'center' : null);
    }
    if (!!changes['isHideLeadingZero']) {
      HtmlElemUtil.setProperty(this.hostRef, CSS_LEADING_ZERO, !!this.isHideLeadingZero ? '""' : null);
    }
    if (!!changes['letterSpacing']) {
      const letterSpacing = this.letterSpacing || -1;
      HtmlElemUtil.setProperty(this.hostRef, CSS_LETTER_SPACING, letterSpacing > 0 ? letterSpacing.toString() + 'px' : null);
    }
  }
  
  // ** Public API **

  // ** Private API **

  private clearCurrValue(): void {
    this.days = -1;
    this.hours = 0;
    this.minutes = 0;
    this.seconds = 0;
  }
  private getLabelDaysObj(labelDays: string | LabelDaysType | null | undefined): LabelDaysType {
    const labelDaysObj: LabelDaysType = { 0: LABEL_DAYS_DEF };
    if (labelDays != null) {
      const typeName = typeof labelDays;
      if (typeName == 'object') {
        const labelDays2 = labelDays as LabelDaysType;
        for(const key in labelDays2) labelDaysObj[key] = labelDays2[key];
      } else if (typeName == 'string') {
        labelDaysObj[0] = labelDays as string;
      }
    }
    return labelDaysObj;
  }
  private updateValueByCurrentDate(currentDate: Date): void {
    // Calculation of parameters: days, hours, minutes and seconds for the current date.
    this.days = 0;
    this.hours = currentDate.getHours();
    this.minutes = currentDate.getMinutes();
    this.seconds = currentDate.getSeconds();
  }
  private updateValueByDistance(distance: number): void {
    // Calculation of parameters: days, hours, minutes and seconds by distance.
    this.days = Math.floor(distance / (1000 * 60 * 60 * 24));
    this.hours = Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    this.minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60));
    this.seconds = Math.floor((distance % (1000 * 60)) / 1000);
  }
  private modifyCurrValueAndSetTimeout = () => {
    // Remove old animation attribute.
    HtmlElemUtil.setAttr(this.renderer, this.hostRef, this.isEvenIteration == true ? 'animat2' : 'animat1', null);
    if (this.isActive && this.isDocumentVisible) {
      const currentDateNum = Date.now();
      let duration = 0;
      let isIncrement = true;
      if (this.startDate != null) {
        const oldDays = this.days;
        const startDateNum = this.startDate.getTime();
        
        isIncrement = startDateNum < currentDateNum;
        // Find the distance between the "current date" and the "start date".
        const distance = (isIncrement ? -1 : 1) * (startDateNum - currentDateNum);
        // Calculation of parameters: days, hours, minutes and seconds by distance.
        this.updateValueByDistance(distance);

        if (oldDays != this.days) {
          this.innLabelDays = this.labelDaysObj[this.days] || this.labelDaysObj[0];
        }
        duration = isIncrement ? (60 - this.seconds) : (this.seconds + 1);
      } else {
        this.updateValueByCurrentDate(new Date(currentDateNum));
        duration = 60 - this.seconds;
      }
      const maxSeconds = duration - 1;

      HtmlElemUtil.setProperty(this.hostRef, CSS_DAYS, this.days.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_HOURS, this.hours.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_MINUTES, this.minutes.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_SECONDS, this.seconds.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_INCREMENT, isIncrement ? '1' : '-1');
      HtmlElemUtil.setProperty(this.hostRef, CCS_MAX_SECONDS, maxSeconds.toString());
      
      this.isEvenIteration = !this.isEvenIteration;
      // Add a new attribute for the new animation.
      HtmlElemUtil.setAttr(this.renderer, this.hostRef, this.isEvenIteration == true ? 'animat2' : 'animat1', '');

      // Start the timer for the next processing.
      this.settimeoutId = window.setTimeout(() => { this.modifyCurrValueAndSetTimeout(); }, duration * 1000); 
    } else {
      this.clearTimeout();
    }
  }
  private clearTimeout(): void {
    if (!this.settimeoutId) {
      window.clearTimeout(this.settimeoutId as number);
      this.settimeoutId = null;
    }
  }
}
