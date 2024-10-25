import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, ElementRef, Input, OnChanges, OnInit, Renderer2, SimpleChanges, 
  ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';

const DEF_LEADING_ZEROS = 2;
const ATTR_IS_ACTIVE = 'is-active';
const CSS_BOX_JUSTIFY_CONTENT = '--dtt-box-justify-content';
const CSS_LEADING_ZERO = '--dtt-leading-zero';
const CSS_LETTER_SPACING = '--dtt-letter-spacing';
const CSS_HOURS = '--dtt-hours';
const CSS_MINUTES = '--dtt-minutes';
const CSS_SECONDS = '--dtt-seconds';

@Component({
  selector: 'app-date-time-timer',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './date-time-timer.component.html',
  styleUrls: ['./date-time-timer.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class DateTimeTimerComponent implements OnChanges, OnInit {
  @Input()
  public isActive: boolean | null | undefined;
  @Input()
  public isAlignCenter: boolean | null | undefined; // together with "isHideLeadingZero"
  @Input()
  public isCountdown: boolean | null | undefined;
  @Input()
  public isHideLeadingZero: boolean | null | undefined;  // together with "isAlignCenter"
  @Input()
  public letterSpacing: number | null | undefined = 0; // in 'px'

  public currValue: Date | null = null;
  public hours: number = 0;
  public minutes: number = 0;
  public seconds: number = 0;
  
  public isEvenIteration: boolean | null = null;
  public settimeoutId: number | null = null;  


  private count = 0; // tmp

  constructor(
    private renderer: Renderer2,
    public hostRef: ElementRef<HTMLElement>,
    private changeDetectorRef: ChangeDetectorRef,
  ) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isActive']) {
      console.log(`isActive: ${this.isActive}`); // #
      if (this.isActive) {
        //this.updateCurrValue(new Date(Date.now())); // #
        //HtmlElemUtil.setProperty(this.hostRef, CSS_HOURS, this.hours.toString()); // #
        //HtmlElemUtil.setProperty(this.hostRef, CSS_MINUTES, this.minutes.toString()); // #
        //this.seconds = 40;
        //HtmlElemUtil.setProperty(this.hostRef, CSS_SECONDS, this.seconds.toString()); // #
  
        this.isEvenIteration = false;
        this.modifyCurrValueAndSetTimeout();
      } else {
        this.isEvenIteration = null;
        this.clearCurrValue();
      }
        
      HtmlElemUtil.setAttr(this.renderer, this.hostRef, ATTR_IS_ACTIVE, !!this.isActive ? '' : null);
    }
    if (!!changes['isAlignCenter']) {
      HtmlElemUtil.setProperty(this.hostRef, CSS_BOX_JUSTIFY_CONTENT, !!this.isAlignCenter ? 'center' : null);
      // justify-content: center
    }
    if (!!changes['isHideLeadingZero']) {
        console.log(`leadingZero: ${!!this.isHideLeadingZero ? JSON.stringify('') : 'null'}`); // #
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

  public log(text: string): void {
    console.log(`log() ${text}`); // #
  }
  // ** Public API **

  private showValue(value: number, isLeadingZero?: boolean | null): string {
    const valueStr = value.toString();
    const len = (!!isLeadingZero ? DEF_LEADING_ZEROS : 0) - valueStr.length;
    return (len > 0 ? Array(len).fill('0').join('') : '') + valueStr;
  }
  private clearCurrValue(): void {
    this.hours = 0;
    this.minutes = 0;
    this.seconds = 0;
    this.currValue = null;

  }
  private updateCurrValue(currValue: Date): number {
    this.hours = currValue.getHours();
    this.minutes = currValue.getMinutes();
    this.seconds = currValue.getSeconds();
    this.currValue = currValue;
    return this.seconds;
  }
  
  private modifyCurrValueAndSetTimeout = () => {
    if (this.isActive && this.count < 2) {
      this.count++
      
      let seconds = this.updateCurrValue(new Date(Date.now()));
      HtmlElemUtil.setProperty(this.hostRef, CSS_HOURS, this.hours.toString());
      HtmlElemUtil.setProperty(this.hostRef, CSS_MINUTES, this.minutes.toString());
      if (this.count == 1) {
        seconds = this.seconds = 0;
      }
      HtmlElemUtil.setProperty(this.hostRef, CSS_SECONDS, (0 != seconds ? this.seconds.toString() : null));

      const duration = (!this.isCountdown ? 60 - seconds : seconds);

      this.isEvenIteration = !this.isEvenIteration;
      
      console.log(`count: ${this.count}, min: ${this.minutes}, sec: ${this.seconds}, duration: ${duration}`
        + `, isEven: ${this.isEvenIteration}`); // #

      this.settimeoutId = window.setTimeout(() => { this.modifyCurrValueAndSetTimeout(); }, duration * 1000); 
    } else if (!this.settimeoutId) {
      window.clearTimeout(this.settimeoutId as number);
      this.settimeoutId = null;
    }
    this.changeDetectorRef.markForCheck();
  }
}
