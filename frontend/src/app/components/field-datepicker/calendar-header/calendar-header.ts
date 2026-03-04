import { CommonModule } from "@angular/common";
import {
    InjectionToken, Component, ViewEncapsulation, ChangeDetectionStrategy, Output, EventEmitter, inject, forwardRef, ChangeDetectorRef
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { DateAdapter, MAT_DATE_FORMATS } from "@angular/material/core";
import { MatCalendarHeader, MatDatepickerIntl, MatCalendar } from "@angular/material/datepicker";

export type CalendarHeaderEvent = {
    activeMonthChanged(): void;
};

export const APP_CALENDAR_HEADER_EVENT = new InjectionToken<CalendarHeaderEvent>("app-calendar-header-event");

@Component({
    selector: "app-calendar-header",
    exportAs: "appCalendarHeader",
    standalone: true,
    imports: [CommonModule, MatButtonModule],
    templateUrl: "./calendar-header.html",
    styleUrl: "./calendar-header.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CalendarHeader<D> extends MatCalendarHeader<D> {

    /** Change the current active month (next/previous month selected).
     * Returns the date (1st of the new active month).
     * This doesn`t imply a change on the selected date.
     */
    @Output() readonly activeMonthChanged: EventEmitter<D> = new EventEmitter<D>();

    private calendarHeaderEvent: CalendarHeaderEvent | null = inject(APP_CALENDAR_HEADER_EVENT, { optional: true });

    constructor() {
        super(
            MatDatepickerIntl,
            inject(forwardRef(() => MatCalendar)), // calendar: MatCalendar<D>,
            inject(DateAdapter<D>, { optional: true }), // dateAdapter: DateAdapter<D>,
            inject(MAT_DATE_FORMATS, { optional: true }), // dateFormats: MatDateFormats,
            inject(ChangeDetectorRef), // changeDetector: ChangeDetectorRef,
        );
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
