import { HttpClient, HttpErrorResponse, HttpHeaders } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import { CreateStreamDto, ModifyStreamDto, StreamDto } from './stream-api.interface';

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
   /*public getStreams(getStreamsDTO: GetStreamsDTO): Promise<StreamsDTO | HttpErrorResponse> {
    const params: HttpParams = HttpParamsUtil.create(getStreamsDTO);
    const url = Uri.appUri('appApi://streams');
    return this.http.get<StreamsDTO | HttpErrorResponse>(url, { params }).toPromise();
  }*/

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

  public modifyStream2(id: number, modifyStreamDto: ModifyStreamDto, file?: File): Promise<StreamDto | HttpErrorResponse | undefined> {
    const data: ModifyStreamDto = {};
    if (!!modifyStreamDto.title) {
      data.title = modifyStreamDto.title;
    }
    if (!!modifyStreamDto.descript) {
      data.descript = modifyStreamDto.descript;
    }
    // if (!!file) {
    //   data.set('logo', file, file.name);
    // }
    if (!!modifyStreamDto.starttime) {
      data.starttime = modifyStreamDto.starttime;
    }
    if (!!modifyStreamDto.source) {
      data.source = modifyStreamDto.source;
    }
    if (!!modifyStreamDto.tags) {
      data.tags = modifyStreamDto.tags;
    }
    
    const url = Uri.appUri(`appApi://streams/${id}`);
    return this.http.put<StreamDto | HttpErrorResponse>(url, data).toPromise();
  }
  /** Delete stream
   * @ route streams/:streamId
   * @ type delete
   * @ params streamId
   * @ required streamId
   * @ access protected
   */
  /*public deleteStream(streamId: string): Promise<StreamDTO | HttpErrorResponse> {
    const url = Uri.appUri(`appApi://streams/${streamId}`);
    return this.http.delete<StreamDTO | HttpErrorResponse>(url).toPromise();
  }*/
}
