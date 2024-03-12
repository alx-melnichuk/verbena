import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { PageInfo, PageInfoUtil } from '../common/page-info';
import { StringDateTime, StringDateTimeUtil } from '../common/string-date-time';
import { AlertService } from '../lib-dialog/alert.service';
import { DateUtil } from '../utils/date.utils';
import { HttpErrorUtil } from '../utils/http-error.util';

import { StreamService } from './stream.service';
import { SearchStreamEventDto, StreamEventDto, StreamEventPageDto, StreamsPeriodDto } from './stream-api.interface';


export const SC_DEFAULT_LIMIT = 12;
export const SC_DELTA_TO_FUTURE = 1;
export const SC_DELTA_TO_PAST = 10;

@Injectable({
  providedIn: 'root'
})
export class StreamCalendarService {
  public calendarMinDate: Date | null = null;
  public calendarMaxDate: Date | null = null;
  
  // ** "Streams Calendar" **
  public calendarMarkedDates: StringDateTime[] = [];
  public calendarLoading = false;
  public calendarMonth: Date = new Date();

  // ** "Streams Event" **
  public eventsOfDay: StreamEventDto[] = [];
  public eventsOfDayLoading = false;
  public eventsOfDayPageInfo: PageInfo = PageInfoUtil.create({ page: 0 });
  public eventsOfDaySelected: Date | null = null;

  constructor(private alertService: AlertService, private streamService: StreamService) {
    this.initDate();
  }

  // ** Public API **

  // ** "Streams Calendar" **

  /** Get calendar information for a period. */
  public getCalendarInfoForPeriod(start: Date, userId?: number): Promise<StreamsPeriodDto[] | HttpErrorResponse | undefined> {
    this.alertService.hide();
    const date: Date = new Date(start);
    date.setHours(0, 0, 0, 0);
    const startMonth = DateUtil.dateFirstDayOfMonth(date);
    if (DateUtil.compare(this.calendarMonth, startMonth) == 0) {
      return Promise.resolve(undefined);
    }
    this.calendarMonth = startMonth;
    const endMonth: Date = DateUtil.dateLastDayOfMonth(startMonth);
    this.calendarLoading = true;
    return this.streamService.getStreamsCalendar({
      start: StringDateTimeUtil.toISO(startMonth),
      finish: StringDateTimeUtil.toISO(endMonth),
      userId
    })
    .then((response: StreamsPeriodDto[] | HttpErrorResponse | undefined) => {
        this.calendarMarkedDates = (response as StreamsPeriodDto[]).map((val) => val.date);
        return response;
    })
    .catch((error: HttpErrorResponse) => {
      this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_get_streams_for_active_period');
      throw error;
    })
    .finally(() => this.calendarLoading = false);
  }

  // ** "Streams Event" **

  /** Clear array of "Streams Event". */
  public clearStreamsEvent(): void {
    this.eventsOfDay = [];
  }
    
  /** Get a list of events for a date. */
  public getListEventsForDate(
    start: Date | null, pageNum: number, userId?: number
  ): Promise<StreamEventPageDto | HttpErrorResponse | undefined> {
    this.alertService.hide();
    const page = pageNum > 0 ? pageNum : (this.eventsOfDayPageInfo.page + 1);
    if (!start || (this.eventsOfDaySelected == start && page > 1 && page > this.eventsOfDayPageInfo.pages)) {
      return Promise.resolve(undefined);
    }
    this.eventsOfDaySelected = start;
    const starttime = StringDateTimeUtil.toISO(start);
    const searchStreamEventDto: SearchStreamEventDto = { userId, starttime, page, limit: SC_DEFAULT_LIMIT };
    this.eventsOfDayLoading = true;
    return this.streamService.getStreamsEvent(searchStreamEventDto)
    .then((response: StreamEventPageDto | HttpErrorResponse | undefined) => {
        const streamEventPageDto = (response as StreamEventPageDto);
        this.eventsOfDayPageInfo = PageInfoUtil.create(streamEventPageDto);
        if (this.eventsOfDayPageInfo.page == 1) {
            this.eventsOfDay = [];
        }
        this.eventsOfDay = this.eventsOfDay.concat(streamEventPageDto.list);
        return response;
    })
    .catch((error: HttpErrorResponse) => {
      this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_get_streams_for_selected_day');
      throw error;
    })
    .finally(() => this.eventsOfDayLoading = false);
  }

  public isShowEvents(calendarDate: Date): boolean {
    return DateUtil.compareYearMonth(calendarDate, this.eventsOfDaySelected) == 0;
  }

  // ** Private API **

  private initDate(): void {
    const today = new Date();
    const timeZoneOffset = -1 * today.getTimezoneOffset();
    let now = new Date(today.getFullYear(), today.getMonth(), today.getDate(), 0, timeZoneOffset, 0, 0);

    const minDateValue = DateUtil.addYear(now, -SC_DELTA_TO_PAST);
    this.calendarMinDate = DateUtil.addDay(minDateValue, -minDateValue.getDate() + 1);

    const maxDateValue = DateUtil.addYear(now, SC_DELTA_TO_FUTURE);
    const daysInMonth = DateUtil.daysInMonth(maxDateValue);
    this.calendarMaxDate = DateUtil.addDay(maxDateValue, daysInMonth - maxDateValue.getDate());
  }
}
