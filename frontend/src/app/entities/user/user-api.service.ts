import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Uri } from 'src/app/utils/uri';
import { LoginUserDto, LoginUserResponseDto, SendUserRegistrationDto, UserDto } from './user-dto';

@Injectable({
  providedIn: 'root',
})
export class UserApiService {
  constructor(private http: HttpClient) {}

  public login(loginUserDto: LoginUserDto): Promise<LoginUserResponseDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://api/login');
    return this.http.post<LoginUserResponseDto | HttpErrorResponse>(url, loginUserDto).toPromise();
  }

  /*public refreshProfileToken(tokenUserDTO: TokenUserDTO): Promise<UserTokensDTO | HttpErrorResponse> {
    const url = Uri.appUri('appApi://profile/token');
    return this.http.post<UserTokensDTO | HttpErrorResponse>(url, tokenUserDTO).toPromise();
  }*/

  /** Registration (step one)
   * @ route user/registration
   * @ type post
   * @ body email, nickname, password, locale
   * @ required email, nickname, password
   * @ access public
   */
  public addUser(sendUserRegistrationDto: SendUserRegistrationDto): Promise<UserDto | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://user/registration');
    return this.http.post<UserDto | HttpErrorResponse>(url, sendUserRegistrationDto).toPromise();
  }
}
