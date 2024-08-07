import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import {
  CreateUserDto, LoginUserDto, LoginUserResponseDto, ModifyProfileDto, RecoveryUserDto, TokenUserDto, UpdatePasswordDto, UserDto, UserDtoUtil, UserProfileDto, UserTokensDto
} from './user-api.interface';
import { HttpObservableUtil } from 'src/app/utils/http-observable.util';

@Injectable({
  providedIn: 'root',
})
export class UserApiService {
  constructor(private http: HttpClient) {}

  public login(loginUserDto: LoginUserDto): Promise<LoginUserResponseDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://login');
    return this.http.post<LoginUserResponseDto | HttpErrorResponse>(url, loginUserDto).toPromise();
  }

  public registration(createUserDto: CreateUserDto): Promise<null | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://registration');
    return this.http.post<null | HttpErrorResponse>(url, createUserDto).toPromise();
  }

  public recovery(recoveryUserDto: RecoveryUserDto): Promise<null | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://recovery');
    return this.http.post<null | HttpErrorResponse>(url, recoveryUserDto).toPromise();
  }
  // TODO del;
  public currentUser(): Promise<UserDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://profiles_current');
    return HttpObservableUtil.toPromise<UserDto>(this.http.get<UserDto | HttpErrorResponse>(url))
    .then((response: UserDto | HttpErrorResponse | undefined) => {
      return UserDtoUtil.new(response as UserDto)
    });
  }

  public isCheckRefreshToken(method: string, url: string): boolean {
    return method === 'POST' && url === Uri.appUri('appApi://token');
  }

  public refreshToken(tokenUserDto: TokenUserDto): Promise<UserTokensDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://token');
    return this.http.post<UserTokensDto | HttpErrorResponse>(url, tokenUserDto).toPromise();
  }

  public logout(): Promise<void | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://logout');
    return this.http.post<void | HttpErrorResponse>(url, null).toPromise();
  }
  
  public modifyProfile(
    id: number, modifyProfileDto: ModifyProfileDto, file?: File | null
  ): Promise<UserProfileDto | HttpErrorResponse | undefined> {
    const formData: FormData = new FormData();
    if (modifyProfileDto.nickname != null) {
      formData.set('nickname', modifyProfileDto.nickname);
    }
    if (modifyProfileDto.email != null) {
      formData.set('email', modifyProfileDto.email);
    }
    if (modifyProfileDto.password != null) {
      formData.set('password', modifyProfileDto.password);
    }
    if (modifyProfileDto.descript != null) {
      formData.set('descript', modifyProfileDto.descript);
    }
    if (file !== undefined) {
      const currFile: File = (file !== null ? file : new File([], "file"));
      formData.set('avatarfile', currFile, currFile.name);
    }
    const headers = new HttpHeaders({ 'enctype': 'multipart/form-data' });
    const url = Uri.appUri(`appApi://users/profile/${id}`);
    return this.http.put<UserProfileDto | HttpErrorResponse>(url, formData, { headers: headers }).toPromise();
  }

  public new_password(updatePasswordDto: UpdatePasswordDto): Promise<UserDto | HttpErrorResponse | undefined> {
    if (!updatePasswordDto.password && !updatePasswordDto.new_password) {
      return Promise.resolve(undefined);
    }
    const url = Uri.appUri("appApi://users_new_password");
    return this.http.put<UserDto | HttpErrorResponse>(url, updatePasswordDto).toPromise();
  }
}
