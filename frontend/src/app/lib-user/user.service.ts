import { HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { UserApiService } from './user-api.service';
import { UserDto, UserTokensDto } from './user-api.interface';

export const ACCESS_TOKEN = 'accessToken_old';
export const REFRESH_TOKEN = 'refreshToken_old';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  public userInfo: UserDto | null = null;
  public userTokensDto: UserTokensDto | null = null;
  
  constructor(private userApiService: UserApiService) {
  }

  public getRefreshToken(): string | null {
    return this.userTokensDto?.refreshToken || null;
  }
  
  public registration(nickname: string, email: string, password: string): Promise<null | HttpErrorResponse | undefined> {
    if (!nickname || !email || !password) {
      return Promise.reject();
    }
    return this.userApiService.registration({ nickname, email, password });
  }

  public recovery(email: string): Promise<null | HttpErrorResponse | undefined> {
    if (!email) {
      return Promise.reject();
    }
    return this.userApiService.recovery({ email });
  }

}
