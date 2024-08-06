import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';

import { HttpParamsUtil } from '../utils/http-params.util';
import { HttpObservableUtil } from '../utils/http-observable.util';

import { UniquenessDto } from './profile-api.interface';

@Injectable({
  providedIn: 'root'
})
export class ProfileApiService {

  constructor(private http: HttpClient) {}

  public uniqueness(nickname: string, email: string): Promise<UniquenessDto | HttpErrorResponse | undefined> {
    if (!nickname && !email) {
      return Promise.resolve(undefined);
    }
    const search = { nickname: (!nickname ? null : nickname), email: (!email ? null : email) };
    const params: HttpParams = HttpParamsUtil.create(search);

    const url = Uri.appUri("appApi://profiles/uniqueness");
    return HttpObservableUtil.toPromise<UniquenessDto>(this.http.get<UniquenessDto | HttpErrorResponse>(url, { params }))
    //return this.http.get<UniquenessDto | HttpErrorResponse>(url, { params }).toPromise();
  }

}
