import {
    ChangeDetectionStrategy, Component, ElementRef, EventEmitter, Input, OnChanges, Output, SimpleChanges,
    ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCalendar, MatCalendarCellClassFunction, MatDatepickerModule } from '@angular/material/datepicker';

import { APP_CALENDAR_HEADER_EVENT, CalendarHeaderComponent } from 'src/app/components/calendar-header/calendar-header.component';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';

import { StreamsPeriodDto } from '../stream-api.interface';

export const PSC_DAY_WITH_STREAMS = 'psc-day-with-streams';
export const PSC_DAY = '---psc-day-';

type MarkedDatesMapTp = { [key: string]: number };

@Component({
    selector: 'app-panel-stream-calendar',
    standalone: true,
    imports: [CommonModule, MatDatepickerModule, CalendarHeaderComponent],
    templateUrl: './panel-stream-calendar.component.html',
    styleUrls: ['./panel-stream-calendar.component.scss', 'panel-stream-calendar-emblem.component.scss'],
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: APP_CALENDAR_HEADER_EVENT, useExisting: PanelStreamCalendarComponent }
    ],
})
export class PanelStreamCalendarComponent implements OnChanges {

    @Input()
    public locale: string | null = null;
    @Input()
    public markedDates: StreamsPeriodDto[] = [];
    @Input()
    public minDate: Date | null = null;
    @Input()
    public maxDate: Date | null = null;
    @Input()
    public selected: Date | null = null;

    @Output()
    readonly changeSelected: EventEmitter<Date | null> = new EventEmitter();
    @Output()
    readonly changeCalendar: EventEmitter<Date> = new EventEmitter();

    @ViewChild('calendar')
    public calendar: MatCalendar<Date> | null = null;

    public startAtDate: Date = new Date(new Date(Date.now()).setHours(0, 0, 0, 0));

    readonly calendarHeader = CalendarHeaderComponent;

    private markedPeriodMap: MarkedDatesMapTp = {};

    constructor(
        public hostRef: ElementRef<HTMLElement>,
    ) {
    }

    public ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['locale'] && !!this.calendar) {
            // Note! This operation allows you to update the value of "periodButtonText" with the new locale.
            this.calendar.activeDate = new Date(this.calendar.activeDate);
        }
        if (!!changes['markedDates'] && !!this.markedDates && !!this.calendar) {
            this.settingPropsByMarkedPeriodMap(this.hostRef, this.markedPeriodMap, false);
            this.markedPeriodMap = this.getMarkedPeriodMap(this.markedDates);
            this.calendar.updateTodaysDate();
            this.settingPropsByMarkedPeriodMap(this.hostRef, this.markedPeriodMap, true);
        }
    }

    // ** type: APP_CALENDAR_HEADER_EVENT **
    public activeMonthChanged = (): void => {
        if (this.calendar != null) {
            this.changeCalendarEmit(this.firstDayOfMonth(this.calendar.activeDate));
        }
    }

    // ** Public API **

    public doViewChanged(calendarView: string): void {
        if (this.calendar != null && 'month' === calendarView) {
            this.changeCalendarEmit(this.firstDayOfMonth(this.calendar.activeDate));
        }
    }

    public doChangeSelected(value: Date | null): void {
        this.changeSelected.emit(value);
    }
    // A function that adds additional CSS classes to date cells.
    public dateClassFn: MatCalendarCellClassFunction<Date> = (date: Date, view: 'month' | 'year' | 'multi-year'): string[] => {
        let result: string[] = [];
        // Only highlight dates inside the "month" view.
        if (view === 'month') {
            const itemLocal = this.getOnlyDate(date);
            const itemCount = itemLocal != null ? this.markedPeriodMap[itemLocal] : null;
            if (itemCount != null) {
                result = [PSC_DAY_WITH_STREAMS, 'psc-day-' + ('00' + date.getDate()).slice(-2)];
            }
        }
        return result;
    };

    // ** Private API **

    private getOnlyDate(value: Date | null): string | null {
        return value == null ? null
            : value.getFullYear() + '-' + ('00' + (value.getMonth() + 1)).slice(-2) + '-' + ('00' + value.getDate()).slice(-2);
    }
    private getMarkedPeriodMap(list: StreamsPeriodDto[]): MarkedDatesMapTp {
        const result: MarkedDatesMapTp = {};
        for (let idx = 0; idx < list.length; idx++) {
            const item: Date | null = new Date(list[idx].date);
            if (!item) continue;
            item.setHours(0, 0, 0, 0);
            const itemLocal = this.getOnlyDate(item);
            if (!itemLocal) continue;
            result[itemLocal] = (result[itemLocal] || 0) + list[idx].count;
        }
        return result;
    }
    private settingPropsByMarkedPeriodMap(el: ElementRef<HTMLElement> | null, markedDatesMap: MarkedDatesMapTp, isSetValue: boolean): void {
        const list: string[] = Object.keys(markedDatesMap);
        for (let idx = 0; idx < list.length; idx++) {
            const value = isSetValue ? markedDatesMap[list[idx]].toString() : null;
            HtmlElemUtil.setProperty(el, PSC_DAY + list[idx].slice(8, 10), value);
        }
    }
    private firstDayOfMonth(value: Date | null): Date | null {
        return value != null ? new Date(value.getFullYear(), value.getMonth(), 1, 0, 0, 0, 0) : null;
    }
    private async changeCalendarEmit(newMonthDate: Date | null): Promise<any> {
        return newMonthDate == null
            ? Promise.resolve()
            : Promise.resolve().then(() => {
                this.changeCalendar.emit(newMonthDate);
            });
    }
}
