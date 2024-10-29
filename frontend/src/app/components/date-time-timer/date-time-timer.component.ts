import {
  ChangeDetectionStrategy, Component, ElementRef, HostListener, Input, OnChanges, OnInit, Renderer2, SimpleChanges, 
  ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';

const ATTR_IS_ACTIVE = 'is-active';
const CSS_BOX_JUSTIFY_CONTENT = '--dtt-box-justify-content';
const CSS_LEADING_ZERO = '--dtt-leading-zero';
const CSS_LETTER_SPACING = '--dtt-letter-spacing';
const CSS_DIRECTION = '--dtt-direction';
const CSS_DAYS = '--dtt-days';
const CSS_HOURS = '--dtt-hours';
const CSS_MINUTES = '--dtt-minutes';
const CSS_SECONDS = '--dtt-seconds';

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
  public futureDate: Date | null | undefined;
  @Input()
  public isActive: boolean | null | undefined;
  @Input()
  public isAlignCenter: boolean | null | undefined; // together with "isHideLeadingZero"
  @Input()
  public isHideLeadingZero: boolean | null | undefined;  // together with "isAlignCenter"
  @Input()
  public letterSpacing: number | null | undefined = 0; // in 'px'

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
    //console.log(`listenerVisibilityChange() isDocumentVisible: ${this.isDocumentVisible}  this.modifyCurrValueAndSetTimeout();`); // #
    this.modifyCurrValueAndSetTimeout();
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['futureDate']) {
      HtmlElemUtil.setProperty(this.hostRef, CSS_DIRECTION, this.futureDate != null ? '-1' : null);
    }
    if (!!changes['isActive']) {
      if (this.isActive) {
        this.isEvenIteration = false;
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
      const letterSpacing = (this.letterSpacing || -1) > 0 ? this.letterSpacing?.toString() + 'px' : null;
      HtmlElemUtil.setProperty(this.hostRef, CSS_LETTER_SPACING, letterSpacing);
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
  private updateValueInBackwardDir(currentDate: Date, futureDate: Date): number {
    const currentDateNum = currentDate.getTime();
    const futureDateNum = futureDate.getTime();
    // Find the distance between now and the countdown date.
    const distance = futureDateNum - currentDateNum;
    // Time calculations for days, hours, minutes and seconds.
    this.days = Math.floor(distance / (1000 * 60 * 60 * 24));
    this.hours = Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    this.minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60));
    this.seconds = Math.floor((distance % (1000 * 60)) / 1000);
    this.currValue = currentDate;
    //console.log(`days: ${this.days}, hours: ${this.hours}, minutes: ${this.minutes}, seconds: ${this.seconds}`); // #
    return this.seconds;
  }
  private clearTimeout(): void {
    if (!this.settimeoutId) {
      window.clearTimeout(this.settimeoutId as number);
      this.settimeoutId = null;
    }
  }
  private modifyCurrValueAndSetTimeout = () => {
    HtmlElemUtil.setAttr(this.renderer, this.hostRef, this.isEvenIteration == true ? 'animat2' : 'animat1', null);
    if (this.isActive && this.isDocumentVisible && this.count < 2) {
      this.count++;
      const now = new Date(Date.now());
      const seconds = this.futureDate != null ?
        this.updateValueInBackwardDir(now, this.futureDate) : this.updateValueInForwardDir(now);

      HtmlElemUtil.setProperty(this.hostRef, CSS_DAYS, this.days.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_HOURS, this.hours.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_MINUTES, this.minutes.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_SECONDS, this.seconds.toString());
      const duration = this.futureDate == null ? 60 - seconds : seconds + 1;
      // const duration = (!this.isCountdown ? 60 - seconds : seconds);
      this.isEvenIteration = !this.isEvenIteration;
      HtmlElemUtil.setAttr(this.renderer, this.hostRef, this.isEvenIteration == true ? 'animat2' : 'animat1', '');
      //console.log(`hours: ${this.hours}, min: ${this.minutes}, sec: ${this.seconds}, duration: ${duration}`
      //  + `, isEven: ${this.isEvenIteration}`); // #
      this.settimeoutId = window.setTimeout(() => { this.modifyCurrValueAndSetTimeout(); }, duration * 1000); 
    } else {
      this.clearTimeout();
    }
  }
}
