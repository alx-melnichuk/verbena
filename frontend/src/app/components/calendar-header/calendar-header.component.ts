import { ChangeDetectionStrategy, Component, ViewEncapsulation, Inject, forwardRef, Optional, ChangeDetectorRef, Output, 
  EventEmitter, InjectionToken } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { DateAdapter, MAT_DATE_FORMATS, MatDateFormats } from '@angular/material/core';
import { MatCalendar, MatCalendarHeader, MatDatepickerIntl } from '@angular/material/datepicker';

export type CalendarHeaderEvent = {
  activeMonthChanged(): void;
};

export const APP_CALENDAR_HEADER_EVENT = new InjectionToken<CalendarHeaderEvent>('app-calendar-header-event');

@Component({
  selector: 'app-calendar-header',
  exportAs: 'appCalendarHeader',
  standalone: true,
  imports: [MatButtonModule],
  templateUrl: './calendar-header.component.html',
  styleUrl: './calendar-header.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CalendarHeaderComponent<D> extends MatCalendarHeader<D>  {

  /** Change the current active month (next/previous month selected).
   * Returns the date (1st of the new active month).
   * This doesn't imply a change on the selected date.
   */
  @Output() readonly activeMonthChanged: EventEmitter<D> = new EventEmitter<D>();

  constructor(
    intl: MatDatepickerIntl,
    @Inject(forwardRef(() => MatCalendar)) calendar: MatCalendar<D>,
    @Optional() dateAdapter: DateAdapter<D>,
    @Optional() @Inject(MAT_DATE_FORMATS) dateFormats: MatDateFormats,
    changeDetectorRef: ChangeDetectorRef,
    @Optional() @Inject(APP_CALENDAR_HEADER_EVENT) private calendarHeaderEvent: CalendarHeaderEvent,
  ) {
    super(intl, calendar, dateAdapter, dateFormats, changeDetectorRef);
  }
  /** Handles user clicks on the previous button. */
  public override previousClicked(): void {
    super.previousClicked();
    this.activeMonthChanged.emit(this.calendar.activeDate);
    this.calendarHeaderEvent?.activeMonthChanged();
  }
  /** Handles user clicks on the next button. */
  public override nextClicked(): void {
    super.nextClicked();
    this.activeMonthChanged.emit(this.calendar.activeDate);
    this.calendarHeaderEvent?.activeMonthChanged();
  }
}
