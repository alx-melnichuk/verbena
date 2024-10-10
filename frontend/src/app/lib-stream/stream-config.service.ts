import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';

import { HttpObservableUtil } from '../utils/http-observable.util';
import { StreamConfigDto } from './stream-config.interface';

@Injectable({
  providedIn: 'root'
})
export class StreamConfigService {
   public streamConfigDto: StreamConfigDto | null = null;

  constructor(private http: HttpClient) {}

  public getConfig(): Promise<StreamConfigDto | HttpErrorResponse | undefined> {
    if (this.streamConfigDto != null) {
      return Promise.resolve({...this.streamConfigDto});
    }
    const url = Uri.appUri('appApi://streams_config');
    return HttpObservableUtil.toPromise<StreamConfigDto>(this.http.get<StreamConfigDto | HttpErrorResponse>(url))
      .then((response: StreamConfigDto | HttpErrorResponse | undefined) => {
        this.streamConfigDto = response as StreamConfigDto;
        return {...this.streamConfigDto};
      });
  }
}
