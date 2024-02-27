import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { PageInfo, PageInfoUtil } from '../common/page-info';
import { StringDate, StringDateTime, StringDateTimeUtil } from '../common/string-date-time';
import { AlertService } from '../lib-dialog/alert.service';
import { DateUtil } from '../utils/date.utils';
import { HttpErrorUtil } from '../utils/http-error.util';

import { StreamService } from './stream.service';
import { SearchStreamEventDto, StreamEventDto, StreamEventListDto, StreamsCalendarDto } from './stream-api.interface';


const CN_DEFAULT_LIMIT = 10;

@Injectable({
  providedIn: 'root'
})
export class StreamCalendarService {

  // ** List of days with events in the month. **
  public streamsCalendarDate: StringDateTime = '';
  // # public selectedDate: StringDate = ''; // moment().add(-1, 'day').format(MOMENT_ISO8601_DATE);
  public streamsCalendarMarkedDates: StringDateTime[] = [];
  // # public selectedDateMarkedDates: string[] = [];
  public streamsCalendarLoading = false;
  // # public selectedDateMarkedDatesLoading = false;
  public streamsCalendarSelectedDate: Date | null = null;
  // # public activePeriodYear: number = moment().year();
  // # public activePeriodMonth: number = moment().month() + 1;
  // # public activeDate: moment.Moment = moment();

  // ** List of events for a date. **
  // # public calendarMiniStreams: StreamDto[] = [];
  public streamsEvent: StreamEventDto[] = [];
  // # public calendarMiniStreamsLoading = false;
  public streamsEventLoading = false;
  // # public calendarMiniStreamsPageInfo: PageInfo = new PageInfo();
  public streamsEventPageInfo: PageInfo = PageInfoUtil.create({ page: 0 });
  public streamsEventStarttime: StringDate = (new Date()).toISOString().slice(0,10);

  constructor(private alertService: AlertService, private streamService: StreamService) {
  }

  // ** Public API **

  // ** List of days with events in the month. **

  //   public getStreamsCalendar(month: number, year: number): Promise<StreamsCalendarDto[] | HttpErrorResponse | undefined>
  public getToday(): Date {
    const today = new Date();
    return new Date(today.getFullYear(), today.getMonth(), today.getDate(), 0, -today.getTimezoneOffset(), 0, 0);
  }

  /** Get a list of days for a new calendar period. */
  public getStreamsCalendarForDate(calendarStart: Date, userId?: number): Promise<StreamsCalendarDto[] | HttpErrorResponse | undefined> {
    this.alertService.hide();

    console.log(`getStreamsCalendarForDate()`); // #
    console.log(`calendarStart:`, calendarStart); // #

    const date: Date = new Date(calendarStart);
    date.setHours(0, 0, 0, 0);
    const date1 = DateUtil.dateFirstDayOfMonth(date);
    console.log(`  date1     :`, date1); // #
    // console.log(`  date1.iso :`, date1.toISOString()); // #
    const startDate: StringDateTime = StringDateTimeUtil.toISO(date1);
    console.log(`  startDate :`, startDate); // #

    if (startDate == this.streamsCalendarDate) {
      return Promise.resolve(undefined);
    }
    this.streamsCalendarDate = startDate;

    const date2: Date = DateUtil.dateLastDayOfMonth(date1);
    console.log(`  date2     :`, date2); // #
    // console.log(`  date2.iso :`, date2.toISOString()); // #
    const finalDate: StringDateTime = StringDateTimeUtil.toISO(date2);
    console.log(`  finalDate :`, finalDate); // #
    
    this.streamsCalendarLoading = true;
    return this.streamService.getStreamsCalendar({ startDate, finalDate, userId })
    .then((response: StreamsCalendarDto[] | HttpErrorResponse | undefined) => {
        this.streamsCalendarMarkedDates = (response as StreamsCalendarDto[]).map((val) => val.date);
        return response;
    })
    .catch((error: HttpErrorResponse) => {
      this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_get_streams_for_active_period');
      throw error;
    })
    .finally(() => {
      this.streamsCalendarLoading = false;
    });
  }
  /*private prepareStreamsCalendar(list: StreamsCalendarDto[]): string[] {
    const result: string[] = [];
    console.log(`prepareStreamsCalendar()`); // #
    const len = list.length;
    for (let index = 0; index < len; index++) {
    //   console.log(`list[${index}].date:`, list[index].date); // #
      const date = new Date(list[index].date);
      console.log(`date    :`, date); // #
      console.log(`date.iso:`, date.toISOString()); // #
      console.log(`date    :`, DateUtil.formatDateTime(date, {
        year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", second: "2-digit",
        timeZoneName: "short"
      })); // #
      const year = date.getFullYear();
      const month = ('00' + (date.getMonth() + 1)).slice(-2);
      const day = ('00' + date.getDate()).slice(-2);
      const str = `${year}-${month}-${day}`;
      console.log(`list[${index}].date:`, list[index].date, ` result.push('${str}');`); // #
      result.push(str);
    }
    return result;
  }*/

  // ** List of events for a date. **

  /** Get a list of short streams for the selected date. */
  public getShortStreamsForDate(
    starttime: StringDateTime, userId?: number
  ): Promise<StreamEventListDto | HttpErrorResponse | undefined> {
    this.alertService.hide();

    let page = (starttime != this.streamsEventStarttime ? 0 : this.streamsEventPageInfo.page) + 1;
    if (!starttime || page > 0 && !PageInfoUtil.checkNextPage(this.streamsEventPageInfo)) {
      return Promise.resolve(undefined);
    }
    const orderDirection = this.streamsEventPageInfo.orderDirection as ('asc' | 'desc' | undefined);
    const searchStreamEventDto: SearchStreamEventDto = { userId, starttime, orderDirection, page, limit: CN_DEFAULT_LIMIT };

    this.streamsEventLoading = true;
    return this.streamService.getStreamsEvent(searchStreamEventDto)
    .then((response: StreamEventListDto | HttpErrorResponse | undefined) => {
        const streamShortListDto = (response as StreamEventListDto);
        this.streamsEventPageInfo = PageInfoUtil.create(streamShortListDto);
        this.streamsEvent = this.streamsEvent.concat(streamShortListDto.list);
        return response;
    })
    .catch((error: HttpErrorResponse) => {
      this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_get_streams_for_selected_day');
      throw error;
    })
    .finally(() => {
      this.streamsEventLoading = false;
    });
  }
/*public setSelectedDateAndGetMiniStreams(userId: string, selectedDate: StringDate): Promise<null | StreamsDTO | HttpErrorResponse> {
  this.alertService.hide();
  if (!selectedDate) {
    return Promise.resolve(null);
  }
  this.selectedDate = selectedDate;
  this.calendarMiniStreamsLoading = true;
  return this.streamService.getStreamsByDate(userId, this.selectedDate, 1)
    .then((response: StreamsDTO | HttpErrorResponse) => {
      const streamsDTO = (response as StreamsDTO);
      this.calendarMiniStreams = streamsDTO.list;
      this.calendarMiniStreamsPageInfo = PageInfoUtil.create(streamsDTO);
      return response;
    })
    .catch((error: HttpErrorResponse) => {
      this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_get_streams_for_selected_day');
      throw error;
    })
    .finally(() => {
      this.calendarMiniStreamsLoading = false;
    });
}*/



/*public isShowCalendarMiniStreams(): boolean {
  return this.isShowCalendarMiniStreamsData(this.activePeriodYear, this.activePeriodMonth, this.selectedDate);
}*/

// If the selected date falls within the active period, then show mini streams.
/*public isShowCalendarMiniStreamsData(year: number, month: number, selectedDate: StringDate): boolean {
  const selected: moment.Moment = moment(selectedDate, MOMENT_ISO8601_DATE);
  const selectYear = selected.year();
  const selectMonth = selected.month() + 1;
  return (selectYear === year && selectMonth === month);
}*/

// Set active period.
/*public setActivePeriod(userId: string, activeDate: moment.Moment): Promise<StreamsCalendarDTO[] | HttpErrorResponse> {
  this.alertService.hide();
  if (!activeDate) {
    return Promise.resolve([]);
  }
  this.activeDate = activeDate;
  const year = this.activePeriodYear = activeDate.year();
  const month = this.activePeriodMonth = activeDate.month() + 1;

  const maket: string = year + '-' + ('0' + month).substr(-2) + '-';
  this.selectedDateMarkedDatesLoading = true;
  return this.streamService.getStreamsCalendar(userId, month, year)
    .then((response: StreamsCalendarDTO[] | HttpErrorResponse) => {
      const streamsCalendarDTO: StreamsCalendarDTO[] = (response as StreamsCalendarDTO[]);
      const markedDates: string[] = (streamsCalendarDTO).map((item) => maket + ('0' + item.day).substr(-2));
      this.selectedDateMarkedDates = markedDates;
      return response;
    })
    .catch((error: HttpErrorResponse) => {
      this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_get_streams_for_active_period');
      throw error;
    })
    .finally(() => {
      this.selectedDateMarkedDatesLoading = false;
    });
}*/
}
