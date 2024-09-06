import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';

import { HttpParamsUtil } from '../utils/http-params.util';
import { HttpObservableUtil } from '../utils/http-observable.util';

import {
 LoginProfileDto, LoginProfileResponseDto, ModifyProfileDto, NewPasswordProfileDto, ProfileDto, ProfileDtoUtil, ProfileTokensDto,
 RecoveryProfileDto,
 RegistrProfileDto,
 TokenDto, UniquenessDto
} from './profile-api.interface';

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

  public registration(registrProfileDto: RegistrProfileDto): Promise<null | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://registration');
    return HttpObservableUtil.toPromise<null>(this.http.post<null | HttpErrorResponse>(url, registrProfileDto));
  }

  public recovery(recoveryProfileDto: RecoveryProfileDto): Promise<null | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://recovery');
    return HttpObservableUtil.toPromise<null>(this.http.post<null | HttpErrorResponse>(url, recoveryProfileDto));
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

    const url = Uri.appUri("appApi://profiles_uniqueness");
    return HttpObservableUtil.toPromise<UniquenessDto>(this.http.get<UniquenessDto | HttpErrorResponse>(url, { params }));
    //return this.http.get<UniquenessDto | HttpErrorResponse>(url, { params }).toPromise();
  }

public modifyProfile(modifyProfileDto: ModifyProfileDto, file?: File | null): Promise<ProfileDto | HttpErrorResponse | undefined> {
    const formData: FormData = new FormData();
    if (modifyProfileDto.nickname != null) {
      formData.set('nickname', modifyProfileDto.nickname);
    }
    if (modifyProfileDto.email != null) {
      formData.set('email', modifyProfileDto.email);
    }
    if (modifyProfileDto.role != null) {
      formData.set('role', modifyProfileDto.role);
    }
    if (modifyProfileDto.descript != null) {
      formData.set('descript', modifyProfileDto.descript);
    }
    if (modifyProfileDto.theme != null) {
      formData.set('theme', modifyProfileDto.theme);
    }
    if (file !== undefined) {
      const currFile: File = (file !== null ? file : new File([], "file"));
      formData.set('avatarfile', currFile, currFile.name);
    }
    const headers = new HttpHeaders({ 'enctype': 'multipart/form-data' });
    const url = Uri.appUri(`appApi://profiles`);
    return HttpObservableUtil.toPromise<ProfileDto>(this.http.put<ProfileDto | HttpErrorResponse>(url, formData, { headers: headers }));
  }

  public newPassword(newPasswordProfileDto: NewPasswordProfileDto): Promise<ProfileDto | HttpErrorResponse | undefined> {
    if (!newPasswordProfileDto.password && !newPasswordProfileDto.newPassword) {
      return Promise.resolve(undefined);
    }
    const url = Uri.appUri("appApi://profiles_new_password");
    return HttpObservableUtil.toPromise<ProfileDto>(this.http.put<ProfileDto | HttpErrorResponse>(url, newPasswordProfileDto));
  }

  public deleteProfileCurrent(): Promise<ProfileDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri("appApi://profiles_current");
    return HttpObservableUtil.toPromise<ProfileDto>(this.http.delete<ProfileDto | HttpErrorResponse>(url));
  }
}
