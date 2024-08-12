import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';

import { HttpParamsUtil } from '../utils/http-params.util';
import { HttpObservableUtil } from '../utils/http-observable.util';

import { LoginProfileDto, LoginProfileResponseDto, ProfileDto, ProfileDtoUtil, ProfileTokensDto, TokenDto, UniquenessDto } from './profile-api.interface';

@Injectable({
  providedIn: 'root'
})
export class ProfileApiService {

  constructor(private http: HttpClient) {}

  public currentProfile(): Promise<ProfileDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://profiles_current');
    return HttpObservableUtil.toPromise<ProfileDto>(this.http.get<ProfileDto | HttpErrorResponse>(url))
    .then((response: ProfileDto | HttpErrorResponse | undefined) => {
      return ProfileDtoUtil.new(response as ProfileDto)
    });
  }

  public login(loginProfileDto: LoginProfileDto): Promise<LoginProfileResponseDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://login');
    return HttpObservableUtil.toPromise<LoginProfileResponseDto>(
      this.http.post<LoginProfileResponseDto | HttpErrorResponse>(url, loginProfileDto));
  }

  public logout(): Promise<void | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://logout');
    return HttpObservableUtil.toPromise<void>(this.http.post<void | HttpErrorResponse>(url, null));
  }

  public isCheckRefreshToken(method: string, url: string): boolean {
    return method === 'POST' && url === Uri.appUri('appApi://token');
  }

  public refreshToken(tokenDto: TokenDto): Promise<ProfileTokensDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://token');
    return HttpObservableUtil.toPromise<ProfileTokensDto>(this.http.post<ProfileTokensDto | HttpErrorResponse>(url, tokenDto));
  }

  public uniqueness(nickname: string, email: string): Promise<UniquenessDto | HttpErrorResponse | undefined> {
    if (!nickname && !email) {
      return Promise.resolve(undefined);
    }
    const search = { nickname: (!nickname ? null : nickname), email: (!email ? null : email) };
    const params: HttpParams = HttpParamsUtil.create(search);

    const url = Uri.appUri("appApi://profiles/uniqueness");
    return HttpObservableUtil.toPromise<UniquenessDto>(this.http.get<UniquenessDto | HttpErrorResponse>(url, { params }));
    //return this.http.get<UniquenessDto | HttpErrorResponse>(url, { params }).toPromise();
  }


  public delete_profile_current(): Promise<ProfileDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri("appApi://profiles_current");
    return HttpObservableUtil.toPromise<ProfileDto>(this.http.delete<ProfileDto | HttpErrorResponse>(url));
  }
}
