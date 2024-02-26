import {
  AfterViewInit, ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, OnDestroy, OnInit, Output,
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
  public markedDates: string[] = [];

  @Output()
  readonly changeSelectedDate: EventEmitter<StringDateTime> = new EventEmitter();
  @Output()
  readonly changeActiveDate: EventEmitter<StringDateTime> = new EventEmitter();

  @ViewChild('calendar')
  public calendar: MatCalendar<Date> | null = null;

  public selectedDate: Date | null = null;
  public minDate: Date; // # = moment().clone().add(-6, 'month').startOf('month');
  public maxDate: Date; // # = moment().clone().add(+6, 'month').endOf('month');
  public startAtDate: Date;
  public activeDate: Date;

  private stateChangesSub: Subscription | undefined;

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
    const value_str = StringDateTimeUtil.toISO(value);  // # dateValue.format(MOMENT_ISO8601_DATE);
    console.log('value_str:', value_str); // #
    this.changeSelectedDate.emit(value_str as StringDateTime);
  }
  // Function used to filter which dates are selectable.
  public dateFilter = (dateValue: Date): boolean => {
    return (!!dateValue);
  }
  // Function that can be used to add custom CSS classes to dates.
  public dateClass: MatCalendarCellClassFunction<Date> = (date: Date, view: 'month' | 'year' | 'multi-year') => {
    // Only highlight dates inside the month view.
    if (view === 'month') {
      const value = StringDateTimeUtil.toISODate(date);
      return (this.markedDates.includes(value) ? 'app-schedule-calendar-day-with-streams' : '');
    }
    return '';
  }

  // ** Private API **

  private changeSateCalendar = (): void => {
    const currActiveDateYYMM = this.activeDate.toISOString().slice(0,7);
    const newActiveDate: Date | null = !!this.calendar ? new Date(this.calendar.activeDate) : null;
    const newActiveDateYYMM = newActiveDate?.toISOString().slice(0,7) || '';
    console.log(`newActiveDate: ${newActiveDate?.toISOString() || ''}`); // #
    if (!!newActiveDate && currActiveDateYYMM != newActiveDateYYMM) {
      this.activeDate = newActiveDate;
      console.log(`this.activeDate = ${newActiveDate.toISOString()}`); // #
      const value_str = StringDateTimeUtil.toISO(this.activeDate);
      this.changeActiveDate.emit(value_str as StringDateTime);
    }
  }
}
