import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import { CreateUserDto, LoginUserDto, LoginUserResponseDto, RecoveryUserDto, TokenUserDto, UserDto, UserTokensDto } from './user-dto';

@Injectable({
  providedIn: 'root',
})
export class UserApiService {
  constructor(private http: HttpClient) {}

  public login(loginUserDto: LoginUserDto): Promise<LoginUserResponseDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://api/login');
    return this.http.post<LoginUserResponseDto | HttpErrorResponse>(url, loginUserDto).toPromise();
  }

  public registration(createUserDto: CreateUserDto): Promise<null | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://api/registration');
    return this.http.post<null | HttpErrorResponse>(url, createUserDto).toPromise();
  }

  public recovery(recoveryUserDto: RecoveryUserDto): Promise<null | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://api/recovery');
    return this.http.post<null | HttpErrorResponse>(url, recoveryUserDto).toPromise();
  }

  public currentUser(): Promise<UserDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://api/users_current');
    return this.http.get<UserDto | HttpErrorResponse>(url).toPromise();
  }

  public isCeckRefreshToken(method: string, url: string): boolean {
    return method === 'POST' && url === Uri.appUri('appApi://api/token');
  }

  public refreshToken(tokenUserDto: TokenUserDto): Promise<UserTokensDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://api/token');
    return this.http.post<UserTokensDto | HttpErrorResponse>(url, tokenUserDto).toPromise();
  }

  public logout(): Promise<void | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://api/logout');
    return this.http.post<void | HttpErrorResponse>(url, null).toPromise();
  }

}
