import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { PageInfo, PageInfoUtil } from '../common/page-info';
import { AlertService } from '../lib-dialog/alert.service';
import { SearchStreamDto, StreamDto, StreamListDto } from './stream-api.interface';
import { StreamService } from './stream.service';
import { HttpErrorUtil } from '../utils/http-error.util';

const CN_DEFAULT_LIMIT = 7; // 10;

@Injectable({
  providedIn: 'root'
})
export class StreamListService {
  // "Future Streams"
  public futureStreamLoading = false;
  public futureStreamsDto: StreamDto[] = [];
  public futurePageInfo: PageInfo = this.createPageInfo();

  // "Past Streams"
  public pastStreamLoading = false;
  public pastStreamsDto: StreamDto[] = [];
  public pastPageInfo: PageInfo = this.createPageInfo();

  constructor(
    private alertService: AlertService,
    private streamService: StreamService,
  ) { 
  }

  // "Future Streams"

  /** Clear array of "Future Stream". */
  public clearFutureStream(): void {
    this.futureStreamsDto = [];
    this.futurePageInfo = this.createPageInfo();
  }
  /** Search for the next page of the "Future Stream". */
  public searchNextFutureStream(userId?: number | undefined): Promise<StreamListDto | HttpErrorResponse | undefined> {
    const pages = this.futurePageInfo.pages;
    const nextPage = this.futurePageInfo.page + 1;
    const isNextPage = ((pages === -1) || (this.futurePageInfo.page !== nextPage && nextPage <= pages));
    if (!isNextPage) {
      return Promise.resolve(undefined);
    }
    let searchStream: SearchStreamDto = {
      userId,
      isFuture: true,
      page: this.futurePageInfo.page + 1,
      limit: this.futurePageInfo.limit
    };
    this.futureStreamLoading = true;
    return this.streamService.getStreams(searchStream)
      .then((response: StreamListDto | HttpErrorResponse | undefined) => {
        const futureStreamListDto = (response as StreamListDto);
        this.futurePageInfo = PageInfoUtil.create(futureStreamListDto);
        this.futureStreamsDto = this.futureStreamsDto.concat(futureStreamListDto.list);
        return response;
      })
      .catch((error: HttpErrorResponse) => {
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_get_future_streams');
        throw error;
      })
      .finally(() => {
        this.futureStreamLoading = false;
      });      
  }

  // "Past Streams"

  /** Clear array of "Past Stream". */
  public clearPastStream(): void {
    this.pastStreamsDto = [];
    this.pastPageInfo = this.createPageInfo();
  }
  /** Search for the next page of the "Past Stream". */
  public searchNextPastStream(userId?: number | undefined): Promise<StreamListDto | HttpErrorResponse | undefined> {
    const pages = this.pastPageInfo.pages;
    const nextPage = this.pastPageInfo.page + 1;
    const isNextPage = ((pages === -1) || (this.pastPageInfo.page !== nextPage && nextPage <= pages));
    if (!isNextPage) {
      return Promise.resolve(undefined);
    }
    let searchStream: SearchStreamDto = {
      userId,
      isFuture: false,
      page: this.pastPageInfo.page + 1,
      limit: this.pastPageInfo.limit
    };
    this.pastStreamLoading = true;
    return this.streamService.getStreams(searchStream)
      .then((response: StreamListDto | HttpErrorResponse | undefined) => {
        const pastStreamListDto = (response as StreamListDto);
        this.pastPageInfo = PageInfoUtil.create(pastStreamListDto);
        this.pastStreamsDto = this.pastStreamsDto.concat(pastStreamListDto.list);
        return response;
      })
      .catch((error: HttpErrorResponse) => {
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_get_past_streams');
        throw error;
      })
      .finally(() => {
        this.pastStreamLoading = false;
      });
  }

  // ** Private API **

  private createPageInfo(): PageInfo {
    return PageInfoUtil.create({ page: 0, limit: CN_DEFAULT_LIMIT });
  }
}
