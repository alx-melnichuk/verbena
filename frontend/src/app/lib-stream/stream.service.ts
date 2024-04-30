import { Injectable } from '@angular/core';
import { Router } from '@angular/router';

import { StreamApiService } from './stream-api.service';
import {
  CreateStreamDto, SearchStreamDto, ModifyStreamDto, StreamDto, StreamListDto, SearchStreamEventDto, StreamEventPageDto, SearchStreamsPeriodDto, StreamsPeriodDto
} from './stream-api.interface';
import { HttpErrorResponse } from '@angular/common/http';
import { Uri } from '../common/uri';
import { ROUTE_VIEW } from '../common/routes';

@Injectable({
  providedIn: 'root'
})
export class StreamService {

   constructor(
    private router: Router,
    private streamApiService: StreamApiService
  ) {
  }
  // ** Public API **

  /** Get streams popular tags */
  /*public getStreamsPopularTags(): Promise<StreamsPopularTagsDTO[] | HttpErrorResponse> {
    return this.streamApiService.getStreamsPopularTags();
  }*/

  /** Get streams calendar */
  /*public getStreamsCalendar(userId: string, month: number, year: number): Promise<StreamsCalendarDTO[] | HttpErrorResponse> {
    return this.streamApiService.getStreamsCalendar(userId, month, year);
  }*/
  public getStreamsCalendar(search: SearchStreamsPeriodDto): Promise<StreamsPeriodDto[] | HttpErrorResponse | undefined> {
    return this.streamApiService.getStreamsPeriod(search);
  }

  /** Get streams
   * - userId (only for groupBy "date")
   * - key (keyword by tag or date, the date should be YYYY-MM-DD)
   * - live (false, true)
   * - starttime (none, past, future)
   * - groupBy (none / tag / date, none by default)
   * - page (number, 1 by default)
   * - limit (number, 10 by default)
   * - orderColumn (starttime / title, starttime by default)
   * - orderDirection (asc / desc, asc by default)
   */
  public getStreams(searchStreamsDTO: SearchStreamDto): Promise<StreamListDto | HttpErrorResponse | undefined> {
    if (!searchStreamsDTO) { return Promise.reject(); }
    return this.streamApiService.getStreams(searchStreamsDTO);
  }
  /*public getStreamsByUser(userId: string, limit?: number, page?: number): Promise<StreamsDTO | HttpErrorResponse> {
    const getStreamsDTO: GetStreamsDTO = {
      userId,
      page: (page || 1), // default = 1;
      limit: (limit || 100), // default = 10;
      orderColumn: 'starttime', // 'starttime' | 'title'; // default = 'starttime';
      orderDirection: 'ASC'     // 'asc' | 'desc'; // default = 'ASC';
    };
    return this.streamApiService.getStreams(getStreamsDTO);
  }*/
  public getStreamsEvent(searchStreamEventDto: SearchStreamEventDto): Promise<StreamEventPageDto | HttpErrorResponse | undefined> {
    if (!searchStreamEventDto) { return Promise.reject(); }
    return this.streamApiService.getStreamsEvent(searchStreamEventDto);
  }
  /*public getStreamsByLive(
    userId: string | null, live: boolean | null, tag: string | null, limit?: number, page?: number
  ): Promise<StreamsDTO | HttpErrorResponse> {
    const getStreamsDTO: GetStreamsDTO = {
      page: (page || 1), // default = 1;
      limit: (limit || 100), // default = 10;
      orderColumn: 'starttime', // 'starttime' | 'title'; // default = 'starttime';
      orderDirection: 'ASC'     // 'asc' | 'desc'; // default = 'ASC';
    };
    if (live != null) {
      getStreamsDTO.live = live;
    }
    if (!!userId) {
      getStreamsDTO.userId = userId;
    }
    if (!!tag) {
      getStreamsDTO.key = tag;
      getStreamsDTO.groupBy = 'tag';
    }
    return this.streamApiService.getStreams(getStreamsDTO);
  }*/

  /** Get stream */
  public getStream(id: number): Promise<StreamDto | HttpErrorResponse | undefined> {
    return this.streamApiService.getStream(id);
  }

  /** Change state stream */
  /*public toggleStreamState(
    streamId: string, streamState: StreamState
  ): Promise<StreamDTO | StreamSetStateForbbidenDTO | HttpErrorResponse> {
    if (streamState === StreamState.waiting) {
      return Promise.reject();
    }
    const streamStateStr: string = streamState.toString();
    return this.streamApiService.toggleStreamState(streamId, { state: (streamStateStr as ToggleStreamState) });
  }*/

  /** Add stream * @ files logo (jpg, png and gif only, 5MB) */
  public createStream(createStreamDto: CreateStreamDto, file?: File): Promise<StreamDto | HttpErrorResponse | undefined> {
    return this.streamApiService.createStream(createStreamDto, file);
  }

  /** Update stream */
  public modifyStream(id: number, modifyStreamDto: ModifyStreamDto, file?: File): Promise<StreamDto | HttpErrorResponse | undefined> {
    return this.streamApiService.modifyStream(id, modifyStreamDto, file);
  }

  /** Delete stream */
  public deleteStream(streamId: number): Promise<void | HttpErrorResponse | undefined> {
    return this.streamApiService.deleteStream(streamId);
  }

  public getLinkForVisitors(streamId: number, isFullPath: boolean): string {
    let prefix = ((isFullPath ? Uri.get('appRoot://') : '') as string);
    if (prefix.slice(-1) === '/') {
      prefix = prefix.slice(0, prefix.length - 1);
    }
    return (!!streamId ? prefix + ROUTE_VIEW + '/' + streamId : '');
  }

  public redirectToStreamViewPage(streamId: number): void {
    // TODO Currently under development.
    // if (!!streamId) {
    //   Promise.resolve().then(() => {
    //     this.router.navigate([ROUTE_VIEW, streamId]);
    //   });
    // }
  }
}
