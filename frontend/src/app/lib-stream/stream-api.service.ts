import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import { HttpParamsUtil } from '../utils/http-params.util';
import {
  CreateStreamDto, SearchStreamDto, ModifyStreamDto, StreamDto, StreamListDto, SearchStreamEventDto, StreamEventPageDto,
  SearchStreamsPeriodDto,
  StreamsPeriodDto
} from './stream-api.interface';
import { StringDate, StringDateTime, StringDateTimeUtil } from '../common/string-date-time';
import { map } from 'rxjs/operators';

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
    return this.http.get<StreamsPopularTagsDTO[] | HttpErrorResponse>(url).toPromise();
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
    return this.http.get<StreamsCalendarDTO[] | HttpErrorResponse>(url).toPromise();
  }*/
  public getStreamsPeriod(search: SearchStreamsPeriodDto): Promise<StreamsPeriodDto[] | HttpErrorResponse | undefined> {
    const params: HttpParams = HttpParamsUtil.create(search);
    const url = Uri.appUri(`appApi://streams_period`);
    return this.http.get<StringDateTime[] | HttpErrorResponse>(url, { params })
      .pipe(map((response) => {
        const result: StreamsPeriodDto[] = [];
        if (Array.isArray(response)) {
          const obj: {[key: string]: number} = {};
          for (let idx = 0; idx < response.length; idx++) {
            const itemDate = StringDateTimeUtil.toDate(response[idx]);
            if (!itemDate) continue;
            itemDate.setHours(0, 0, 0, 0);
            const itemLocal = StringDateTimeUtil.toISOLocal(itemDate);

            obj[itemLocal] = (obj[itemLocal] || 0) + 1;
          }
          const keys = Object.keys(obj);
          for (let i = 0; i < keys.length; i++) {
            result.push({ date: keys[i], count: obj[keys[i]] });
          }
        }
        return result;
      }))
      .toPromise();
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
    return this.http.get<StreamListDto | HttpErrorResponse>(url, { params }).toPromise();
  }
  
  public getStreamsEvent(searchStreamEventDto: SearchStreamEventDto): Promise<StreamEventPageDto | HttpErrorResponse | undefined> {
    const params: HttpParams = HttpParamsUtil.create(searchStreamEventDto);
    const url = Uri.appUri('appApi://streams_events');
    return this.http.get<StreamEventPageDto | HttpErrorResponse>(url, { params }).toPromise();
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
    return this.http.get<StreamDto | HttpErrorResponse>(url).toPromise();
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
   /*public toggleStreamState(
     streamId: string, data: ToggleStreamStateDTO
   ): Promise<StreamDTO | StreamSetStateForbbidenDTO | HttpErrorResponse> {
    const url = Uri.appUri(`appApi://streams/toggle/${streamId}`);
    return this.http.put<StreamDTO | HttpErrorResponse>(url, data).toPromise();
  }*/

  /** Add stream
   * @ route streams
   * @ type post
   * @ body title, description, starttime, tags (array stringify, 4 max)
   * @ files logo (jpg, png and gif only, 5MB)
   * @ required title, description
   * @ access protected
   */
  public createStream(createStreamDto: CreateStreamDto, file?: File): Promise<StreamDto | HttpErrorResponse | undefined> {
    const formData: FormData = new FormData();
    formData.set('title', createStreamDto.title);
    if (!!createStreamDto.descript) {
      formData.set('descript', createStreamDto.descript);
    }
    if (!!createStreamDto.starttime) {
      formData.set('starttime', createStreamDto.starttime);
    }
    if (!!createStreamDto.source) {
      formData.set('source', createStreamDto.source);
    }
    if (createStreamDto.tags.length > 0) {
        formData.set('tags', JSON.stringify(createStreamDto.tags));
    }
    if (!!file) {
      formData.set('logofile', file, file.name);
    }
    const url = Uri.appUri(`appApi://streams`);
    return this.http.post<StreamDto | HttpErrorResponse>(url, formData).toPromise();
  }

  /** Update stream
   * @ route streams/:streamId
   * @ type put
   * @ params streamId
   * @ body title, descript, starttime, tags (array stringify, 3 max)
   * @ required streamId
   * @ access protected
   */
  public modifyStream(id: number, modifStreamDto: ModifyStreamDto, file?: File | null): Promise<StreamDto | HttpErrorResponse | undefined> {
    const formData: FormData = new FormData();
    if (!!modifStreamDto.title) {
      formData.set('title', modifStreamDto.title);
    }
    if (!!modifStreamDto.descript) {
      formData.set('descript', modifStreamDto.descript);
    }
    if (file !== undefined) {
      const currFile: File = (file !== null ? file : new File([], "file"));
      formData.set('logofile', currFile, currFile.name);
    }

    if (!!modifStreamDto.starttime) {
      formData.set('starttime', modifStreamDto.starttime);
    }
    if (!!modifStreamDto.source) {
      formData.set('source', modifStreamDto.source);
    }
    if (!!modifStreamDto.tags) {
      formData.set('tags', JSON.stringify(modifStreamDto.tags));
    }
    const headers = new HttpHeaders({ 'enctype': 'multipart/form-data' });
    const url = Uri.appUri(`appApi://streams/${id}`);
    return this.http.put<StreamDto | HttpErrorResponse>(url, formData, { headers: headers }).toPromise();
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
    return this.http.delete<void | HttpErrorResponse>(url).toPromise();
  }
}
