import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import {
  CreateUserDto, LoginUserDto, LoginUserResponseDto, RecoveryUserDto, TokenUserDto, UserDto, UserDtoUtil, UserTokensDto
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

  public currentUser(): Promise<UserDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://users_current');
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

}
