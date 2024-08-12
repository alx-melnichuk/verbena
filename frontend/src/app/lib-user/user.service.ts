import { HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { UserApiService } from './user-api.service';
import { ModifyProfileDto, UpdatePasswordDto, UserDto, UserProfileDto, UserTokensDto } from './user-api.interface';

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

  public modifyProfile(
    id: number, modifyProfileDto: ModifyProfileDto, file?: File | null
  ): Promise<UserProfileDto | HttpErrorResponse | undefined> {
    return this.userApiService.modifyProfile(id, modifyProfileDto, file);
  }
  
  public new_password(updatePasswordDto: UpdatePasswordDto): Promise<UserDto | HttpErrorResponse | undefined> {
    return this.userApiService.new_password(updatePasswordDto);
  }

}
