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
            const itemDate = StringDateTimeUtil.to_date(response[idx]);
            if (!itemDate) continue;
            const item: StringDate = StringDateTimeUtil.toISODate(itemDate);
            obj[item] = (obj[item] || 0) + 1;
          }
          const keys = Object.keys(obj);
          for (let i = 0; i < keys.length; i++) {
            result.push({ date: keys[i], count: obj[keys[i]] });
          }
        }
        return result;
      }))
      .toPromise();
    /*return new Promise<StreamsPeriodDto[]>(
        (resolve: (value: StreamsPeriodDto[] | PromiseLike<StreamsPeriodDto[]>) => void, reject: (reason?: any) => void) => {
          setTimeout(() => {
            if (!!search) {
              resolve(this.streamsPeriodDto(search.start));
            } else {
              reject('err');
            }
          }, 500);
        });*/
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
  public modifyStream(id: number, modifyStreamDto: ModifyStreamDto, file?: File): Promise<StreamDto | HttpErrorResponse | undefined> {
    const formData: FormData = new FormData();
    if (!!modifyStreamDto.title) {
      formData.set('title', modifyStreamDto.title);
    }
    if (!!modifyStreamDto.descript) {
      formData.set('descript', modifyStreamDto.descript);
    }
    if (!!file) {
      formData.set('logofile', file, file.name);
    //   const file2: File = new File([], "foo.txt");
    //   formData.set('logofile', file2, file2.name);
    }

    if (!!modifyStreamDto.starttime) {
      formData.set('starttime', modifyStreamDto.starttime);
    }
    if (!!modifyStreamDto.source) {
      formData.set('source', modifyStreamDto.source);
    }
    if (!!modifyStreamDto.tags) {
      formData.set('tags', JSON.stringify(modifyStreamDto.tags));
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

  /*private streamsPeriodDto(startDate: StringDateTime): StreamsPeriodDto[] {
    const date = new Date(startDate);
    // console.log(`@startDate:`, startDate); // #
    const i = date.getMonth() % 2 !== 0 ? 0 : 2;
    const date1 = new Date(date.getFullYear(), date.getMonth(), 10 + i, date.getHours(), date.getMinutes(), date.getSeconds());
    const date2 = new Date(date.getFullYear(), date.getMonth(), 15 + i, date.getHours(), date.getMinutes(), date.getSeconds());
    const date3 = new Date(date.getFullYear(), date.getMonth(), 20 + i, date.getHours(), date.getMinutes(), date.getSeconds());
    const date4 = new Date(date.getFullYear(), date.getMonth(), 25 + i, date.getHours(), date.getMinutes(), date.getSeconds());
    const list: StreamsPeriodDto[] = [
      { "date": date1.toISOString(), "day": 10, "count": 1 }, // '2024-02-10T11:08:05.553Z+0200'
      { "date": date2.toISOString(), "day": 15, "count": 1 }, // '2024-02-15T11:08:05.553Z+0200'
      { "date": date3.toISOString(), "day": 20, "count": 1 }, // '2024-02-20T11:08:05.553Z+0200'
      { "date": date4.toISOString(), "day": 25, "count": 1 }, // '2024-02-25T11:08:05.553Z+0200'
    ];
    return list;
  }*/
}
