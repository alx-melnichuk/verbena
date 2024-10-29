import {
  ChangeDetectionStrategy, Component, ElementRef, HostListener, Input, OnChanges, OnInit, Renderer2, SimpleChanges, 
  ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';

const LABEL_DAYS_DEF = 'days';
const ATTR_IS_ACTIVE = 'is-active';
const CSS_BOX_JUSTIFY_CONTENT = '--dtt-box-justify-content';
const CSS_LEADING_ZERO = '--dtt-leading-zero';
const CSS_LETTER_SPACING = '--dtt-letter-spacing';
const CSS_RATE = '--dtt-rate';
const CCS_MAX_SECONDS = '--dtt-max-seconds'
const CSS_DAYS = '--dtt-days';
const CSS_HOURS = '--dtt-hours';
const CSS_MINUTES = '--dtt-minutes';
const CSS_SECONDS = '--dtt-seconds';

export type LabelDaysType = { [key: number]: string };

/*
  <!-- Timer countdown ascending -->
  <app-date-time-timer [isActive]="true">
  </app-date-time-timer>

  <!-- The timer counts down to the specified date 'startDate'. -->
  <app-date-time-timer
    [isActive]="true"
    [labelDays]="'days'"
    [startDate]="startDate">
  </app-date-time-timer>

  <!-- The timer counts down to the specified date 'startDate'.
       The daily marker changes depending on the number of days remaining. -->
  <app-date-time-timer
    [isActive]="true"
    [labelDays]="{ '0': 'days', '1': 'only days', '2': 'total days' }"
    [startDate]="startDate">
  </app-date-time-timer>
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
export class DateTimeTimerComponent implements OnChanges, OnInit {
  @Input()
  public startDate: Date | null | undefined;
  @Input()
  public isActive: boolean | null | undefined;
  @Input()
  public isAlignCenter: boolean | null | undefined; // together with "isHideLeadingZero"
  @Input()
  public isHideLeadingZero: boolean | null | undefined;  // together with "isAlignCenter"
  @Input()
  public labelDays: string | LabelDaysType | null | undefined;
  @Input()
  public letterSpacing: number | null | undefined = 0; // in 'px'

  public innLabelDays: string = '';
  public labelDaysObj: LabelDaysType = this.getLabelDaysObj(null);

  private currValue: Date | null = null;
  private days: number = 0;
  private hours: number = 0;
  private minutes: number = 0;
  private seconds: number = 0;
  
  private isEvenIteration: boolean = false;
  private settimeoutId: number | null = null;
  private isDocumentVisible: boolean = true;
  private count = 0;
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
    if (!!changes['startDate']) {
      HtmlElemUtil.setProperty(this.hostRef, CSS_RATE, this.startDate != null ? '-1' : null);
    }
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
  
  ngOnInit(): void {
  }

  // ** Public API **

  // ** Public API **

  private clearCurrValue(): void {
    this.days = 0;
    this.hours = 0;
    this.minutes = 0;
    this.seconds = 0;
    this.currValue = null;

  }
  private updateValueInForwardDir(currentDate: Date): number {
    this.days = 0;
    this.hours = currentDate.getHours();
    this.minutes = currentDate.getMinutes();
    this.seconds = currentDate.getSeconds();
    this.currValue = currentDate;
    return this.seconds;
  }
  private updateValueInBackwardDir(currentDate: Date, startDate: Date): number {
    const currentDateNum = currentDate.getTime();
    const startDateNum = startDate.getTime();
    // Find the distance between now and the countdown date.
    const distance = startDateNum - currentDateNum;
    // Time calculations for days, hours, minutes and seconds.
    this.days = Math.floor(distance / (1000 * 60 * 60 * 24));
    this.hours = Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    this.minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60));
    this.seconds = Math.floor((distance % (1000 * 60)) / 1000);
    this.currValue = currentDate;

    return this.seconds;
  }
  private clearTimeout(): void {
    if (!this.settimeoutId) {
      window.clearTimeout(this.settimeoutId as number);
      this.settimeoutId = null;
    }
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
  private modifyCurrValueAndSetTimeout = () => {
    HtmlElemUtil.setAttr(this.renderer, this.hostRef, this.isEvenIteration == true ? 'animat2' : 'animat1', null);
    if (this.isActive && this.isDocumentVisible && this.count < 3) {
      this.count++;
      const now = new Date(Date.now());
      let seconds = 0;
      if (this.startDate != null) {
        const oldDays = this.days;
        seconds = this.updateValueInBackwardDir(now, this.startDate)
        if (oldDays != this.days) {
          this.innLabelDays = this.labelDaysObj[this.days] || this.labelDaysObj[0];
        }
      } else {
        seconds = this.updateValueInForwardDir(now);
      }
      HtmlElemUtil.setProperty(this.hostRef, CSS_DAYS, this.days.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_HOURS, this.hours.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_MINUTES, this.minutes.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_SECONDS, this.seconds.toString());
      const maxSeconds = this.startDate == null ? 59 - seconds : seconds;
      HtmlElemUtil.setProperty(this.hostRef, CCS_MAX_SECONDS, maxSeconds.toString());
      const duration = this.startDate == null ? 60 - seconds : seconds + 1;
      this.isEvenIteration = !this.isEvenIteration;
      HtmlElemUtil.setAttr(this.renderer, this.hostRef, this.isEvenIteration == true ? 'animat2' : 'animat1', '');

      this.settimeoutId = window.setTimeout(() => { this.modifyCurrValueAndSetTimeout(); }, duration * 1000); 
    } else {
      this.clearTimeout();
    }
  }
}
