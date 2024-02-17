import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { OrderDirection, PageData } from '../common/page-data';
import { UserService } from '../entities/user/user.service';
import { AlertService } from '../lib-dialog/alert.service';
import { SearchStreamDto, StreamListDto } from './stream-api.interface';
import { StreamService } from './stream.service';
import { HttpErrorUtil } from '../utils/http-error.util';

const CN_DEFAULT_LIMIT = 7; // 10;

@Injectable({
  providedIn: 'root'
})
export class StreamListService {
  // "Future Streams"
  public futureStreamLoading = false;
  public futureStreamsDto: StreamListDto = this.emptyStreamListDto();

  // "Past Streams"
  public pastStreamLoading = false;
  public pastStreamsDto: StreamListDto = this.emptyStreamListDto();


  constructor(
    private alertService: AlertService,
    // private profileService: ProfileService,
    // private userService: UserService,
    private streamService: StreamService,
  ) { 
    console.log(`StreamListService()`); // # 
  }

  public clearFutureStream(): void {
    this.futureStreamsDto = this.emptyStreamListDto();
  }

  public clearPastStream(): void {
    this.pastStreamsDto = this.emptyStreamListDto();
  }

  public getFutureStream(userId: number, pageData: PageData): Promise<StreamListDto | HttpErrorResponse | undefined> {
    const orderColumn = (pageData.orderColumn as ('starttime' | 'title' | undefined));
    // const userId = (this.profileService.getUserId() as string);
    const getStreamsDto: SearchStreamDto = {
      userId,
      isFuture: true, // starttime: 'future',
      page: (pageData.page || 1), // default = 1;
      limit: (pageData.limit > 0 ? pageData.limit : CN_DEFAULT_LIMIT), // default = 10; min = 1, max = 100;
      orderColumn: (orderColumn || 'starttime'),
      orderDirection: (pageData.orderDirection === OrderDirection.desc ? 'desc' : 'asc')
    };
    console.log(`StreamListService.getFutureStream()  getStreamsDto: `, getStreamsDto); // #
    this.futureStreamsDto = this.emptyStreamListDto();
    this.futureStreamLoading = true;
    return this.streamService.getStreams(getStreamsDto)
      .then((response: StreamListDto | HttpErrorResponse | undefined) => {
        this.futureStreamsDto = (response as StreamListDto);
        console.log(`StreamListService.getFutureStream()  this.futureStreamsDto = `, this.futureStreamsDto); // #
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

  public getPastStream(userId: number, pageData: PageData): Promise<StreamListDto | HttpErrorResponse | undefined> {
    const orderColumn = (pageData.orderColumn as ('starttime' | 'title' | undefined));
    // const userId = (this.profileService.getUserId() as string);
    const getStreamsDto: SearchStreamDto = {
      userId,
      isFuture: false, // starttime: 'past',
      page: (pageData.page || 1), // default = 1;
      limit: (pageData.limit > 0 ? pageData.limit : CN_DEFAULT_LIMIT), // default = 10; min = 1, max = 100;
      orderColumn: (orderColumn || 'starttime'),
      orderDirection: (pageData.orderDirection === OrderDirection.desc ? 'desc' : 'asc')
    };
    this.pastStreamsDto = this.emptyStreamListDto();
    this.pastStreamLoading = true;
    return this.streamService.getStreams(getStreamsDto)
      .then((response: StreamListDto | HttpErrorResponse | undefined) => {
        this.pastStreamsDto = (response as StreamListDto);
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

  // ** Private API **

  private emptyStreamListDto(): StreamListDto {
    return { list: [], limit: 0, count: 0, page: 0, pages: -1 };
  }
}
