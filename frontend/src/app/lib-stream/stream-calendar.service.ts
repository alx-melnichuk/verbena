import { Injectable } from '@angular/core';
import { AlertService } from '../lib-dialog/alert.service';
import { StreamService } from './stream.service';
import { SearchStreamEventDto, StreamEventDto, StreamEventListDto, StringDate, StringDateTime } from './stream-api.interface';
import { PageInfo, PageInfoUtil } from '../common/page-info';
import { HttpErrorUtil } from '../utils/http-error.util';
import { HttpErrorResponse } from '@angular/common/http';

const CN_DEFAULT_LIMIT = 10;

@Injectable({
  providedIn: 'root'
})
export class StreamCalendarService {

  // "Panel Calendar"
  // # public selectedDate: StringDate = moment().add(-1, 'day').format(MOMENT_ISO8601_DATE);
  // # public selectedDateMarkedDates: string[] = [];
  // # public selectedDateMarkedDatesLoading = false;
  // # public activePeriodYear: number = moment().year();
  // # public activePeriodMonth: number = moment().month() + 1;
  // # public activeDate: moment.Moment = moment();

  // "Panel Calendar for StreamShort list"
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

  /** For the selected date, get a list of streams for short streams. */
  public setSelectedDateAndGetShortStreams(
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
    // getStreamsEventByDate(userId, this.selectedDate, 1)
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
  // For the selected date, get a list of streams for mini streams.
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
