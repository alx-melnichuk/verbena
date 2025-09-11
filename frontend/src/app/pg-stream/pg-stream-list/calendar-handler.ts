import { HttpErrorResponse } from "@angular/common/http";

import { PageInfo, PageInfoUtil } from "src/app/common/page-info";
import { StringDateTime } from "src/app/common/string-date-time";
import { StreamEventDto, StreamEventPageDto, StreamsPeriodDto } from "src/app/lib-stream/stream-api.interface";
import { StreamService } from "src/app/lib-stream/stream.service";
import { DateUtil } from "src/app/utils/date.utils";
import { StringDateTimeUtil } from "src/app/utils/string-date-time.util";

export const SC_DEFAULT_LIMIT = 12;
export const SC_DELTA_TO_FUTURE = 10;
export const SC_DELTA_TO_PAST = 10;

export class CalendarHandler {
    public calendarMinDate: Date;
    public calendarMaxDate: Date;

    // ** "Streams Calendar" **
    public calendarMarkedDates: StreamsPeriodDto[] = [];
    public calendarLoading = false;
    public calendarMonth: Date = new Date();

    // ** "Streams Event" **
    public eventsOfDay: StreamEventDto[] = [];
    public eventsOfDayLoading = false;
    public eventsOfDayPageInfo: PageInfo = PageInfoUtil.create({ page: 0 });
    public eventsOfDaySelected: Date | null = null;

    constructor(
        private streamService: StreamService,
    ) {
        const today = new Date();
        const timeZoneOffset = -1 * today.getTimezoneOffset();
        // Get the current day, only the date from 0 hours 0 minutes 0 seconds.
        let now = new Date(today.getFullYear(), today.getMonth(), today.getDate(), 0, timeZoneOffset, 0, 0);
        // Get the minimum date of choice in the calendar (today - 10 years).
        const minDateValue = DateUtil.addYear(now, -SC_DELTA_TO_PAST);
        this.calendarMinDate = DateUtil.addDay(minDateValue, -minDateValue.getDate() + 1);
        // Get the maximum date of choice in the calendar (today + 10 years).
        const maxDateValue = DateUtil.addYear(now, SC_DELTA_TO_FUTURE);
        const daysInMonth = DateUtil.daysInMonth(maxDateValue);
        this.calendarMaxDate = DateUtil.addDay(maxDateValue, daysInMonth - maxDateValue.getDate());
    }

    // ** Public API **

    // ** "Streams Calendar" **

    /** Get calendar information for a period. */
    public async getCalendarInfoForPeriod(
        start: Date, isRequired: boolean, userId?: number
    ): Promise<StreamsPeriodDto[] | HttpErrorResponse | undefined> {
        const date: Date = new Date(start);
        date.setHours(0, 0, 0, 0);
        const startMonth = DateUtil.dateFirstDayOfMonth(date);
        if (!isRequired && DateUtil.compare(this.calendarMonth, startMonth) == 0) {
            return Promise.resolve(undefined);
        }
        this.calendarMonth = startMonth;
        const endMonth: Date = DateUtil.dateLastDayOfMonth(startMonth);
        endMonth.setHours(23, 59, 59, 0);
        this.calendarLoading = true;
        try {
            const search = {
                start: startMonth.toISOString(),
                finish: endMonth.toISOString(),
                userId
            };
            const response = await this.streamService.getStreamsCalendar(search);
            this.calendarMarkedDates = this.convertStringDateTimeToStreamsPeriodDto(response as StringDateTime[]);
            return this.calendarMarkedDates;
        } finally {
            this.calendarLoading = false;
        }
    }

    // ** "Streams Event" **

    /** Clear array of "Streams Event". */
    public clearStreamsEvent(): void {
        this.eventsOfDay = [];
    }
    /** Get a list of events for a date. */
    public async getListEventsForDate(
        start: Date | null, pageNum: number, userId?: number
    ): Promise<StreamEventPageDto | HttpErrorResponse | undefined> {
        const page = pageNum > 0 ? pageNum : (this.eventsOfDayPageInfo.page + 1);
        if (!start || (this.eventsOfDaySelected == start && page > 1 && page > this.eventsOfDayPageInfo.pages)) {
            return Promise.resolve(undefined);
        }
        this.eventsOfDaySelected = start;
        const starttime = start.toISOString();
        this.eventsOfDayLoading = true;
        try {
            const response = await this.streamService.getStreamsEvent({ userId, starttime, page, limit: SC_DEFAULT_LIMIT });
            const streamEventPageDto = (response as StreamEventPageDto);
            this.eventsOfDayPageInfo = PageInfoUtil.create(streamEventPageDto);
            if (this.eventsOfDayPageInfo.page == 1) {
                this.eventsOfDay = [];
            }
            this.eventsOfDay = this.eventsOfDay.concat(streamEventPageDto.list);
            return response;
        } finally {
            this.eventsOfDayLoading = false;
        }
    }

    public isShowEvents(calendarDate: Date): boolean {
        return DateUtil.compareYearMonth(calendarDate, this.eventsOfDaySelected) == 0;
    }

    // ** Private API **

    private getOnlyDate(value: Date | null): string | null {
        return value == null ? null
            : value.getFullYear() + '-' + ('00' + (value.getMonth() + 1)).slice(-2) + '-' + ('00' + value.getDate()).slice(-2);
    }
    private convertStringDateTimeToStreamsPeriodDto(response: StringDateTime[]): StreamsPeriodDto[] {
        const result: StreamsPeriodDto[] = [];
        if (Array.isArray(response)) {
            const obj: { [key: string]: number } = {};
            for (let idx = 0; idx < response.length; idx++) {
                const itemDate: Date | null = StringDateTimeUtil.toDate(response[idx]);
                if (!itemDate) continue;
                itemDate.setHours(0, 0, 0, 0);
                const itemLocal = this.getOnlyDate(itemDate);
                if (!itemLocal) continue;
                obj[itemLocal] = (obj[itemLocal] || 0) + 1;
            }
            const keys = Object.keys(obj);
            for (let i = 0; i < keys.length; i++) {
                result.push({ date: keys[i], count: obj[keys[i]] });
            }
        }
        return result;
    }
}