import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import { HttpParamsUtil } from '../utils/http-params.util';
import {
  CreateStreamDto, SearchStreamDto, ModifyStreamDto, StreamDto, StreamListDto, SearchStreamEventDto, StreamEventListDto,
  StreamEventDto, StreamsCalendarDto, SearchStreamsCalendarDto
} from './stream-api.interface';
import { StringDateTime } from '../common/string-date-time';

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
  public getStreamsCalendar(search: SearchStreamsCalendarDto): Promise<StreamsCalendarDto[] | HttpErrorResponse | undefined> {
    const params: HttpParams = HttpParamsUtil.create(search);
    // const url = Uri.appUri(`appApi://streams/calendar/${month}/${year}`);
    // return this.http.get<StreamsCalendarDto[] | HttpErrorResponse>(url, { params }).toPromise();
    return new Promise<StreamsCalendarDto[]>(
        (resolve: (value: StreamsCalendarDto[] | PromiseLike<StreamsCalendarDto[]>) => void, reject: (reason?: any) => void) => {
          setTimeout(() => {
            if (!!search) {
              resolve(this.streamsCalendarDto(search.startDate));
            } else {
              reject('err');
            }
          }, 500);
        });
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
  
  public getStreamsEvent(searchStreamEventDto: SearchStreamEventDto): Promise<StreamEventListDto | HttpErrorResponse | undefined> {
    const params: HttpParams = HttpParamsUtil.create(searchStreamEventDto);
    // const url = Uri.appUri('appApi://streams-event');
    // return this.http.get<StreamEventListDto | HttpErrorResponse>(url, { params }).toPromise();
    return new Promise<StreamEventListDto>(
      (resolve: (value: StreamEventListDto | PromiseLike<StreamEventListDto>) => void, reject: (reason?: any) => void) => {
        setTimeout(() => {
          if (!!searchStreamEventDto.startDate) {
            resolve(this.getStreamEventListDto(searchStreamEventDto));
          } else {
            reject('err');
          }
        }, 500);
      });
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

  private getStreamEventListDto(search: SearchStreamEventDto): StreamEventListDto {
    const tr = ' trip to ';
    const avt = '- E.Allen';
    const dir = '/assets/images/trip_';
    const listVal: StreamEventDto[] = [
      { "id":  1, "userId": 10, "title": `01${tr}cyprus 0${avt}` , "logo": `${dir}cyprus01.jpg`, "starttime": "2024-01-10T09:08:05.553Z" },
      { "id":  2, "userId": 10, "title": `02${tr}greece 0${avt}` , "logo": `${dir}greece01.jpg`, "starttime": "2024-01-20T09:08:05.553Z" },
      { "id":  3, "userId": 10, "title": `03${tr}spain 0${avt}`  , "logo": `${dir}spain01.jpg` , "starttime": "2024-02-10T09:08:05.553Z" },
      { "id":  4, "userId": 10, "title": `04${tr}cyprusA 0${avt}`, "logo": `${dir}cyprus02.jpg`, "starttime": "2024-02-20T09:08:05.553Z" },
      { "id":  5, "userId": 10, "title": `05${tr}greeceA 0${avt}`, "logo": `${dir}greece02.jpg`, "starttime": "2024-03-10T09:08:05.553Z" },
      { "id":  6, "userId": 10, "title": `06${tr}spainA 0${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-03-20T09:08:05.553Z" },
      { "id":  7, "userId": 10, "title": `07${tr}cyprus 0${avt}` , "logo": `${dir}cyprus01.jpg`, "starttime": "2024-04-10T09:08:05.553Z" },
      { "id":  8, "userId": 10, "title": `08${tr}greece 0${avt}` , "logo": `${dir}greece01.jpg`, "starttime": "2024-04-20T09:08:05.553Z" },
      { "id":  9, "userId": 10, "title": `09${tr}spain 0${avt}`  , "logo": `${dir}spain01.jpg` , "starttime": "2024-05-10T09:08:05.553Z" },
      { "id": 10, "userId": 10, "title": `10${tr}cyprusA 1${avt}`, "logo": `${dir}cyprus02.jpg`, "starttime": "2024-05-20T09:08:05.553Z" },
      { "id": 11, "userId": 10, "title": `11${tr}greeceA 1${avt}`, "logo": `${dir}greece02.jpg`, "starttime": "2024-06-10T09:08:05.553Z" },
      { "id": 12, "userId": 10, "title": `12${tr}spainA 1${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-06-20T09:08:05.553Z" },
      { "id": 13, "userId": 10, "title": `13${tr}cyprus 1${avt}` , "logo": `${dir}cyprus01.jpg`, "starttime": "2024-01-11T09:08:05.553Z" },
      { "id": 14, "userId": 10, "title": `14${tr}greece 1${avt}` , "logo": `${dir}greece01.jpg`, "starttime": "2024-01-21T09:08:05.553Z" },
      { "id": 15, "userId": 10, "title": `15${tr}spain 1${avt}`  , "logo": `${dir}spain01.jpg` , "starttime": "2024-02-11T09:08:05.553Z" },
      { "id": 16, "userId": 10, "title": `16${tr}cyprusA 1${avt}`, "logo": `${dir}cyprus02.jpg`, "starttime": "2024-02-21T09:08:05.553Z" },
      { "id": 17, "userId": 10, "title": `17${tr}greeceA 1${avt}`, "logo": `${dir}greece02.jpg`, "starttime": "2024-03-21T09:08:05.553Z" },
      { "id": 18, "userId": 10, "title": `18${tr}spainA 1${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-03-21T09:08:05.553Z" },
      { "id": 19, "userId": 10, "title": `19${tr}cyprus 1${avt}` , "logo": `${dir}cyprus01.jpg`, "starttime": "2024-04-11T09:08:05.553Z" },
      { "id": 20, "userId": 10, "title": `20${tr}greece 2${avt}` , "logo": `${dir}greece01.jpg`, "starttime": "2024-04-21T09:08:05.553Z" },
      { "id": 21, "userId": 10, "title": `21${tr}spain 2${avt}`  , "logo": `${dir}spain01.jpg` , "starttime": "2024-05-11T09:08:05.553Z" },
      { "id": 22, "userId": 10, "title": `22${tr}cyprusA 2${avt}`, "logo": `${dir}cyprus02.jpg`, "starttime": "2024-05-12T09:08:05.553Z" },
      { "id": 23, "userId": 10, "title": `23${tr}greeceA 2${avt}`, "logo": `${dir}greece02.jpg`, "starttime": "2024-06-11T09:08:05.553Z" },
      { "id": 24, "userId": 10, "title": `24${tr}spainA 2${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-06-21T09:08:05.553Z" },
      { "id": 25, "userId": 10, "title": `25${tr}cyprus 2${avt}` , "logo": `${dir}cyprus01.jpg`, "starttime": "2024-01-12T09:08:05.553Z" },
      { "id": 26, "userId": 10, "title": `26${tr}greece 2${avt}` , "logo": `${dir}greece01.jpg`, "starttime": "2024-01-22T09:08:05.553Z" },
      { "id": 27, "userId": 10, "title": `27${tr}spain 2${avt}`  , "logo": `${dir}spain01.jpg` , "starttime": "2024-02-12T09:08:05.553Z" },
      { "id": 28, "userId": 10, "title": `28${tr}cyprusA 2${avt}`, "logo": `${dir}cyprus02.jpg`, "starttime": "2024-02-22T09:08:05.553Z" },
      { "id": 29, "userId": 10, "title": `29${tr}greeceA 2${avt}`, "logo": `${dir}greece02.jpg`, "starttime": "2024-03-12T09:08:05.553Z" },
      { "id": 30, "userId": 10, "title": `30${tr}spainA 3${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-03-22T09:08:05.553Z" },
      { "id": 31, "userId": 10, "title": `31${tr}cyprus 3${avt}` , "logo": `${dir}cyprus01.jpg`, "starttime": "2024-04-12T09:08:05.553Z" },
      { "id": 32, "userId": 10, "title": `32${tr}greece 3${avt}` , "logo": `${dir}greece01.jpg`, "starttime": "2024-04-22T09:08:05.553Z" },
      { "id": 33, "userId": 10, "title": `33${tr}spain 3${avt}`  , "logo": `${dir}spain01.jpg` , "starttime": "2024-05-12T09:08:05.553Z" },
      { "id": 34, "userId": 10, "title": `34${tr}cyprusA 3${avt}`, "logo": `${dir}cyprus02.jpg`, "starttime": "2024-05-22T09:08:05.553Z" },
      { "id": 35, "userId": 10, "title": `35${tr}greeceA 3${avt}`, "logo": `${dir}greece02.jpg`, "starttime": "2024-06-12T09:08:05.553Z" },
      { "id": 36, "userId": 10, "title": `36${tr}spainA 3${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-06-22T09:08:05.553Z" },
      { "id": 37, "userId": 10, "title": `37${tr}cyprus 3${avt}` , "logo": `${dir}cyprus01.jpg`, "starttime": "2024-01-12T09:08:05.553Z" },
      { "id": 38, "userId": 10, "title": `38${tr}greece 3${avt}` , "logo": `${dir}greece01.jpg`, "starttime": "2024-01-22T09:08:05.553Z" },
      { "id": 39, "userId": 10, "title": `39${tr}spain 3${avt}`  , "logo": `${dir}spain01.jpg` , "starttime": "2024-02-12T09:08:05.553Z" },
      { "id": 40, "userId": 10, "title": `40${tr}cyprusA 4${avt}`, "logo": `${dir}cyprus02.jpg`, "starttime": "2024-02-22T09:08:05.553Z" },
      { "id": 41, "userId": 10, "title": `41${tr}greeceA 4${avt}`, "logo": `${dir}greece02.jpg`, "starttime": "2024-03-22T09:08:05.553Z" },
      { "id": 42, "userId": 10, "title": `42${tr}spainA 4${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-03-22T09:08:05.553Z" },
      { "id": 43, "userId": 10, "title": `43${tr}cyprus 4${avt}` , "logo": `${dir}cyprus01.jpg`, "starttime": "2024-04-12T09:08:05.553Z" },
      { "id": 44, "userId": 10, "title": `44${tr}greece 4${avt}` , "logo": `${dir}greece01.jpg`, "starttime": "2024-04-22T09:08:05.553Z" },
      { "id": 45, "userId": 10, "title": `45${tr}spain 4${avt}`  , "logo": `${dir}spain01.jpg` , "starttime": "2024-05-12T09:08:05.553Z" },
      { "id": 46, "userId": 10, "title": `46${tr}cyprusA 4${avt}`, "logo": `${dir}cyprus02.jpg`, "starttime": "2024-05-12T09:08:05.553Z" },
      { "id": 47, "userId": 10, "title": `47${tr}greeceA 4${avt}`, "logo": `${dir}greece02.jpg`, "starttime": "2024-06-12T09:08:05.553Z" },
      { "id": 48, "userId": 10, "title": `48${tr}spainA 4${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-06-22T09:08:05.553Z" },
      { "id": 49, "userId": 10, "title": `49${tr}spainA 4${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-06-22T09:08:05.553Z" },
      { "id": 50, "userId": 10, "title": `50${tr}spainA 5${avt}` , "logo": `${dir}spain02.jpg` , "starttime": "2024-06-22T09:08:05.553Z" },
    ];
    const count = listVal.length;
    const page = search.page || 1;
    const limit = search.limit || 9;
    const pages = Math.ceil(count / limit);
    const start = (page -1 ) * limit;
    const finishVal = page * limit;
    const finish = finishVal > count ? count : finishVal;
    const list = listVal.slice(start, finish);
    return { list, limit, count, page, pages };
  }
  private streamsCalendarDto(startDate: StringDateTime): StreamsCalendarDto[] {
    const date = new Date(startDate);
    // console.log(`@startDate:`, startDate); // #
    const i = date.getMonth() % 2 !== 0 ? 0 : 2;
    const date1 = new Date(date.getFullYear(), date.getMonth(), 10 + i, date.getHours(), date.getMinutes(), date.getSeconds());
    const date2 = new Date(date.getFullYear(), date.getMonth(), 15 + i, date.getHours(), date.getMinutes(), date.getSeconds());
    const date3 = new Date(date.getFullYear(), date.getMonth(), 20 + i, date.getHours(), date.getMinutes(), date.getSeconds());
    const date4 = new Date(date.getFullYear(), date.getMonth(), 25 + i, date.getHours(), date.getMinutes(), date.getSeconds());
    const list: StreamsCalendarDto[] = [
      { "date": date1.toISOString(), "day": 10, "count": 1 }, // '2024-02-10T11:08:05.553Z+0200'
      { "date": date2.toISOString(), "day": 15, "count": 1 }, // '2024-02-15T11:08:05.553Z+0200'
      { "date": date3.toISOString(), "day": 20, "count": 1 }, // '2024-02-20T11:08:05.553Z+0200'
      { "date": date4.toISOString(), "day": 25, "count": 1 }, // '2024-02-25T11:08:05.553Z+0200'
    ];
    return list;
  }
}
