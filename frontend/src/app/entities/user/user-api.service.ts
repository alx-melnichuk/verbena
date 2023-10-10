import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Uri } from 'src/app/utils/uri';
import { CreateUserDto, LoginUserDto, LoginUserResponseDto, UserDto } from './user-dto';

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

  /*public refreshProfileToken(tokenUserDTO: TokenUserDTO): Promise<UserTokensDTO | HttpErrorResponse> {
    const url = Uri.appUri('appApi://profile/token');
    return this.http.post<UserTokensDTO | HttpErrorResponse>(url, tokenUserDTO).toPromise();
  }*/
}
