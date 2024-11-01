import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import { HttpParamsUtil } from '../utils/http-params.util';
import {
  SearchStreamDto, StreamDto, StreamListDto, SearchStreamEventDto, StreamEventPageDto, SearchStreamsPeriodDto,
  UpdateStreamFileDto, StreamState
} from './stream-api.interface';
import { StringDateTime } from '../common/string-date-time';
import { HttpObservableUtil } from '../utils/http-observable.util';

@Injectable({
  providedIn: 'root'
})
export class StreamApiService {

constructor(private http: HttpClient) {
  }

  /** Get streams popular tags
   * @ route streams/popular/tags
   * @ type get
   * @ access public
   */
  /*public getStreamsPopularTags(): Promise<StreamsPopularTagsDTO[] | HttpErrorResponse> {
    const url = Uri.appUri('appApi://streams/popular/tags');
    return HttpObservableUtil.toPromise<StreamsPopularTagsDTO[]>(this.http.get<StreamsPopularTagsDTO[] | HttpErrorResponse>(url));
  }*/

  /** Get streams calendar
   * @ streams/calendar/:userId/:month/:year
   * @ type get
   * @ params userId, month, year
   * @ required userId, month, year
   * @ access public
   */
  /*public getStreamsCalendar(userId: string, month: number, year: number): Promise<StreamsCalendarDTO[] | HttpErrorResponse> {
    const url = Uri.appUri(`appApi://streams/calendar/${userId}/${month}/${year}`);
    return HttpObservableUtil.toPromise<StreamsCalendarDTO[]>(this.http.get<StreamsCalendarDTO[] | HttpErrorResponse>(url));
  }*/
  public getStreamsPeriod(search: SearchStreamsPeriodDto): Promise<StringDateTime[] | HttpErrorResponse | undefined> {
    const params: HttpParams = HttpParamsUtil.create(search);
    const url = Uri.appUri(`appApi://streams_period`);
    return HttpObservableUtil.toPromise<StringDateTime[]>(this.http.get<StringDateTime[] | HttpErrorResponse>(url, { params }));
  }
  /** Get streams
   * @ route streams
   * @ example streams?groupBy=date&userId=385e0469-7143-4915-88d0-f23f5b27ed28/9/2022&orderColumn=title&orderDirection=desc&live=true
   * @ type get
   * @ query pagination (optional):
   * - userId (only for groupBy "date")
   * - key (keyword by tag or date, the date should be YYYY-MM-DD)
   * - live (false, true)
   * - starttime (none, past, future)
   * - groupBy (none / tag / date, none by default)
   * - page (number, 1 by default)
   * - limit (number, 10 by default)
   * - orderColumn (starttime / title, starttime by default)
   * - orderDirection (asc / desc, asc by default)
   * @ access public
   */
   public getStreams(searchStreamDto: SearchStreamDto): Promise<StreamListDto | HttpErrorResponse | undefined> {
    const params: HttpParams = HttpParamsUtil.create(searchStreamDto);
    const url = Uri.appUri('appApi://streams');
    return HttpObservableUtil.toPromise<StreamListDto>(this.http.get<StreamListDto | HttpErrorResponse>(url, { params }));
  }
  
  public getStreamsEvent(searchStreamEventDto: SearchStreamEventDto): Promise<StreamEventPageDto | HttpErrorResponse | undefined> {
    const params: HttpParams = HttpParamsUtil.create(searchStreamEventDto);
    const url = Uri.appUri('appApi://streams_events');
    return HttpObservableUtil.toPromise<StreamEventPageDto>(this.http.get<StreamEventPageDto | HttpErrorResponse>(url, { params }));
  }

  /** Get stream
   * @ route streams/:streamId
   * @ type get
   * @ params streamId
   * @ required streamId
   * @ access public
   */
  public getStream(id: number): Promise<StreamDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri(`appApi://streams/${id}`);
    return HttpObservableUtil.toPromise<StreamDto>(this.http.get<StreamDto | HttpErrorResponse>(url));
  }

  /** Change state stream
   * @ route streams/toggle/:streamId
   * @ example streams/toggle/385e0469-7143-4915-88d0-f23f5b27ed36
   * @ type put
   * @ params streamId
   * @ body state ['preparing' | 'started' | 'stopped' | 'paused']
   * @ required streamId
   * @ access protected
   */
   public toggleStreamState(
     streamId: number, state: StreamState
   ): Promise<StreamDto /*| StreamSetStateForbbidenDTO*/ | HttpErrorResponse> {
    const url = Uri.appUri(`appApi://streams/toggle/${streamId}`);
    return HttpObservableUtil.toPromise<StreamDto>(this.http.put<StreamDto | HttpErrorResponse>(url, { state: state }));
  }

  /** Add stream
   * @ route streams
   * @ type post
   * @ body title, description, starttime, tags (array stringify, 4 max)
   * @ files logo (jpg, png and gif only, 5MB)
   * @ required title, description
   * @ access protected
   */
  public createStream(updateStreamFileDto: UpdateStreamFileDto): Promise<StreamDto | HttpErrorResponse | undefined> {
    const formData: FormData = new FormData();
    formData.set('title', updateStreamFileDto.title || '');
    if (!!updateStreamFileDto.descript) {
      formData.set('descript', updateStreamFileDto.descript);
    }
    if (!!updateStreamFileDto.starttime) {
      formData.set('starttime', updateStreamFileDto.starttime);
    }
    if (!!updateStreamFileDto.source) {
      formData.set('source', updateStreamFileDto.source);
    }
    if (!!updateStreamFileDto.tags) {
        formData.set('tags', JSON.stringify(updateStreamFileDto.tags));
    }
    if (!!updateStreamFileDto.logoFile) {
      formData.set('logofile', updateStreamFileDto.logoFile, updateStreamFileDto.logoFile.name);
    }
    const url = Uri.appUri(`appApi://streams`);
    return HttpObservableUtil.toPromise<StreamDto>(this.http.post<StreamDto | HttpErrorResponse>(url, formData));
  }

  /** Update stream
   * @ route streams/:streamId
   * @ type put
   * @ params streamId
   * @ body title, descript, starttime, tags (array stringify, 3 max)
   * @ required streamId
   * @ access protected
   */
  public modifyStream(id: number, updateStreamFileDto: UpdateStreamFileDto): Promise<StreamDto | HttpErrorResponse | undefined> {
    const formData: FormData = new FormData();
    if (updateStreamFileDto.title != null) {
      formData.set('title', updateStreamFileDto.title);
    }
    if (updateStreamFileDto.descript != null) {
      formData.set('descript', updateStreamFileDto.descript);
    }
    if (updateStreamFileDto.logoFile !== undefined) {
      const currFile: File = (updateStreamFileDto.logoFile !== null ? updateStreamFileDto.logoFile : new File([], "file"));
      formData.set('logofile', currFile, currFile.name);
    }
    if (!!updateStreamFileDto.starttime) {
      formData.set('starttime', updateStreamFileDto.starttime);
    }
    if (!!updateStreamFileDto.source) {
      formData.set('source', updateStreamFileDto.source);
    }
    if (!!updateStreamFileDto.tags) {
      formData.set('tags', JSON.stringify(updateStreamFileDto.tags));
    }
    const headers = new HttpHeaders({ 'enctype': 'multipart/form-data' });
    const url = Uri.appUri(`appApi://streams/${id}`);
    return HttpObservableUtil.toPromise<StreamDto>(this.http.put<StreamDto | HttpErrorResponse>(url, formData, { headers: headers }));
  }

  /** Delete stream
   * @ route streams/:streamId
   * @ type delete
   * @ params streamId
   * @ required streamId
   * @ access protected
   */
  public deleteStream(streamId: number): Promise<void | HttpErrorResponse | undefined> {
    const url = Uri.appUri(`appApi://streams/${streamId}`);
    return HttpObservableUtil.toPromise<void>(this.http.delete<void | HttpErrorResponse>(url));
  }
}
