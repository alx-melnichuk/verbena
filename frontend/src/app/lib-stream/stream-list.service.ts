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
  public futurePageInfo: PageInfo = PageInfoUtil.create({ page: 0, limit: CN_DEFAULT_LIMIT });

  // "Past Streams"
  public pastStreamLoading = false;
  public pastStreamsDto: StreamDto[] = [];
  public pastPageInfo: PageInfo = PageInfoUtil.create({ page: 0, limit: CN_DEFAULT_LIMIT });

  constructor(
    private alertService: AlertService,
    private streamService: StreamService,
  ) { 
    console.log(`StreamListService()`); // # 
  }

  public clearFutureStream(): void {
    this.futureStreamsDto = [];
  }

  public clearPastStream(): void {
    this.pastStreamsDto = [];
  }

  public searchNextFutureStream(userId: number): Promise<StreamListDto | HttpErrorResponse | undefined> {
    if (!this.checkNextPage(this.futurePageInfo)) {
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
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'my_streams.error_get_future_streams');
        throw error;
      })
      .finally(() => {
        this.futureStreamLoading = false;
      });      
  
  }

  public searchNextPastStream(userId: number): Promise<StreamListDto | HttpErrorResponse | undefined> {
    if (!this.checkNextPage(this.pastPageInfo)) {
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
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'my_streams.error_get_past_streams');
        throw error;
      })
      .finally(() => {
        this.pastStreamLoading = false;
      });      
  }

  public checkNextPage(pageInfo: PageInfo): boolean {
    const nextPageInfo: PageInfo = PageInfoUtil.create({ page: pageInfo.page + 1 });
    return PageInfoUtil.checkNextPage(this.pastPageInfo, nextPageInfo);
  }

  // ** Private API **

}
