import {
  AfterViewInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, Input, OnChanges, OnDestroy, OnInit, Output,
  SimpleChanges,
  ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCalendar, MatCalendarCellClassFunction, MatDatepickerModule } from '@angular/material/datepicker';
import { MatNativeDateModule } from '@angular/material/core';
import { StringDateTime, StringDateTimeUtil } from 'src/app/common/string-date-time';
import { DateUtil } from 'src/app/utils/date.utils';
import { Subscription } from 'rxjs';

export const PSC_DELTA_TO_FUTURE = 1;
export const PSC_DELTA_TO_PAST = 20;
export const PSC_DAY_WITH_STREAMS = 'psc-day-with-streams';

@Component({
  selector: 'app-panel-stream-calendar',
  standalone: true,
  imports: [CommonModule, MatNativeDateModule, MatDatepickerModule],
  templateUrl: './panel-stream-calendar.component.html',
  styleUrls: ['./panel-stream-calendar.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamCalendarComponent implements OnInit, OnChanges, AfterViewInit, OnDestroy {

  @Input()
  public selected: StringDateTime | null; // # = moment().format(MOMENT_ISO8601_DATE);
  @Input()
  public markedDates: StringDateTime[] = [];

  @Output()
  readonly changeSelectedDate: EventEmitter<Date | null> = new EventEmitter();
  @Output()
  readonly changeCalendar: EventEmitter<Date> = new EventEmitter();

  @ViewChild('calendar')
  public calendar: MatCalendar<Date> | null = null;

  public selectedDate: Date | null = null;
  public minDate: Date; // # = moment().clone().add(-6, 'month').startOf('month');
  public maxDate: Date; // # = moment().clone().add(+6, 'month').endOf('month');
  public startAtDate: Date;
  public activeDate: Date;

  private stateChangesSub: Subscription | undefined;
  private markedDatesStr: string[] = [];

  constructor() {
    const today = new Date();
    const timeZoneOffset = -1 * today.getTimezoneOffset();
    // console.log(`today: `, today.toISOString());
    let now = new Date(today.getFullYear(), today.getMonth(), today.getDate(), 0, timeZoneOffset, 0, 0);
    // console.log(`nowISO : `, now.toISOString());
    // console.log(`now    : `, now);
    this.selectedDate = now;
    this.selected = StringDateTimeUtil.toISO(this.selectedDate);

    this.startAtDate = now;
    this.activeDate = now;
    
    const minDateValue = DateUtil.addYear(now, -PSC_DELTA_TO_PAST);
    this.minDate = DateUtil.addDay(minDateValue, -minDateValue.getDate() + 1);
    // console.log(`minDate: `, this.minDate.toISOString());
    // console.log(`minDate: `, this.minDate);

    const maxDateValue = DateUtil.addYear(now, PSC_DELTA_TO_FUTURE);
    const daysInMonth = DateUtil.daysInMonth(maxDateValue);
    this.maxDate = DateUtil.addDay(maxDateValue, daysInMonth - maxDateValue.getDate());
    // console.log(`maxDate: `, this.maxDate.toISOString());
    // console.log(`maxDate: `, this.maxDate);
  }
  
  public ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['selected'] && !!this.selected) {
      this.selectedDate = StringDateTimeUtil.to_date(this.selected);
    }
    if (!!changes['markedDates'] && !!this.markedDates && !!this.calendar) {
    //   console.log(`this.markedDates:`, this.markedDates); // #
      this.markedDatesStr = this.markedDates.map(val => this.getInfoDate(new Date(val)));
      this.calendar.updateTodaysDate();
    }
  }

  public ngOnInit(): void {
  }

  public ngAfterViewInit(): void {
    if (!!this.calendar) {
      this.activeDate = new Date(this.calendar.activeDate);
      this.stateChangesSub = this.calendar.stateChanges
        .subscribe((value) => this.changeSateCalendar());
    }
  }

  public ngOnDestroy(): void {
    this.stateChangesSub?.unsubscribe();
  }
  
  // ** Public API **

  public doChangeSelected(value: Date | null): void {
    if (!value) { return; }
    this.changeSelectedDate.emit(value);
  }
  // Function that can be used to add custom CSS classes to dates.
  public dateClassFn: MatCalendarCellClassFunction<Date> = (date: Date, view: 'month' | 'year' | 'multi-year') => {
    let result: string = '';
    // Only highlight dates inside the month view.
    if (view === 'month') {
      const value = this.getInfoDate(date);
      if (this.markedDatesStr.includes(value)) {
        result = PSC_DAY_WITH_STREAMS;
      }
    }
    return result;
  };

  // ** Private API **

  private changeSateCalendar = (): void => {
    if (!this.calendar) {
      return;
    }
    const newActiveDate: Date = new Date(this.calendar.activeDate);
    const newActiveDateYearMonth = this.getInfoDate(newActiveDate).slice(0, 7);
    // console.log(`newActiveDate:`, newActiveDate); // #

    const currActiveDateYearMonth = this.getInfoDate(this.activeDate).slice(0, 7);
    if (!!newActiveDate && currActiveDateYearMonth != newActiveDateYearMonth) {
      const activeDate = new Date(newActiveDate.getFullYear(), newActiveDate.getMonth(), 1, 0, 0, 0, 0);
      this.activeDate = activeDate;
    //   console.log(`activeDate:`, activeDate); // #
      this.changeCalendar.emit(new Date(activeDate));
    }
  }

  private getInfoDate(date: Date): string {
    const year = date.getFullYear();
    const month = ('00' + (date.getMonth() + 1)).slice(-2);
    const day = ('00' + date.getDate()).slice(-2);
    return `${year}-${month}-${day}`;
  }
}
