import { Injectable } from '@angular/core';
import { Router } from '@angular/router';

import { StreamApiService } from './stream-api.service';
import { ModifyStreamDto, StreamDto } from './stream-api.interface';
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

  /** Get streams popular tags
   */
  /*public getStreamsPopularTags(): Promise<StreamsPopularTagsDTO[] | HttpErrorResponse> {
    return this.streamApiService.getStreamsPopularTags();
  }*/

  /** Get streams calendar
   */
  /*public getStreamsCalendar(userId: string, month: number, year: number): Promise<StreamsCalendarDTO[] | HttpErrorResponse> {
    return this.streamApiService.getStreamsCalendar(userId, month, year);
  }*/

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
  /*public getStreams(getStreamsDTO: GetStreamsDTO): Promise<StreamsDTO | HttpErrorResponse> {
    if (!getStreamsDTO) { return Promise.reject(); }
    return this.streamApiService.getStreams(getStreamsDTO);
  }*/
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
  /*public getStreamsByDate(userId: string, selectedDate: string, page: number): Promise<StreamsDTO | HttpErrorResponse> {
    const getStreamsDTO: GetStreamsDTO = {
      userId,
      key: selectedDate, // '2021-04-27',
      groupBy: 'date', // 'none' | 'tag' | 'date'; // default = 'none';
      page: (page || 1), // default = 1;
      limit: 100, // default = 10;
      orderColumn: 'starttime', // 'starttime' | 'title'; // default = 'starttime';
      orderDirection: 'ASC'     // 'asc' | 'desc'; // default = 'ASC';
    };
    return this.streamApiService.getStreams(getStreamsDTO);
  }*/
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

  /** Get stream
   */
  public getStream(id: number): Promise<StreamDto | HttpErrorResponse | undefined> {
    return this.streamApiService.getStream(id);
  }

  /** Change state stream
   */
  /*public toggleStreamState(
    streamId: string, streamState: StreamState
  ): Promise<StreamDTO | StreamSetStateForbbidenDTO | HttpErrorResponse> {
    if (streamState === StreamState.waiting) {
      return Promise.reject();
    }
    const streamStateStr: string = streamState.toString();
    return this.streamApiService.toggleStreamState(streamId, { state: (streamStateStr as ToggleStreamState) });
  }*/

  /** Add stream
   * @ files logo (jpg, png and gif only, 5MB)
   */
  /*public addStream(addStreamDTO: AddStreamDTO, file?: File): Promise<StreamDTO | HttpErrorResponse> {
    return this.streamApiService.addStream(addStreamDTO, file);
  }*/

  /** Update stream
   */
  public modifyStream(id: number, modifyStreamDto: ModifyStreamDto, file?: File): Promise<StreamDto | HttpErrorResponse | undefined> {
    return this.streamApiService.modifyStream(id, modifyStreamDto, file);
  }

  public modifyStreamUpload(desc: string, file: File): Promise<string | HttpErrorResponse | undefined> {
    return this.streamApiService.modifyStreamUpload(desc, file);
  }

  public send_form(modifyStreamDto: ModifyStreamDto, file?: File): Promise<any | HttpErrorResponse | undefined> {
    return this.streamApiService.send_form(modifyStreamDto, file);
  }

  /** Delete stream
   */
  /*public deleteStream(streamId: string): Promise<StreamDTO | HttpErrorResponse> {
    return this.streamApiService.deleteStream(streamId);
  }*/

  public getLinkForVisitors(streamId: number, isFullPath: boolean): string {
    let prefix = ((isFullPath ? Uri.get('appRoot://') : '') as string);
    if (prefix.slice(-1) === '/') {
      prefix = prefix.slice(0, prefix.length - 1);
    }
    return (!!streamId ? prefix + ROUTE_VIEW + '/' + streamId : '');
  }

  public redirectToStreamViewPage(streamId: string): void {
    if (!!streamId) {
      Promise.resolve().then(() => {
        this.router.navigate([ROUTE_VIEW, streamId]);
      });
    }
  }
}
