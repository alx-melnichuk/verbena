import { HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { UserApiService } from './user-api.service';
import { LoginUserResponseDto, ModifyProfileDto, UpdatePasswordDto, UserDto, UserProfileDto, UserTokensDto } from './user-api.interface';

export const ACCESS_TOKEN = 'accessToken_old';
export const REFRESH_TOKEN = 'refreshToken_old';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  public userInfo: UserDto | null = null;
  public userTokensDto: UserTokensDto | null = null;
  
  constructor(private userApiService: UserApiService) {
    this.userTokensDto = this.getUserTokensDtoFromLocalStorage();
  }

  public hasAccessTokenInLocalStorage(): boolean {
    return !!localStorage.getItem(ACCESS_TOKEN);
  }

  public isExistRefreshToken(): boolean {
    return !!this.userTokensDto?.refreshToken;
  }

  public getAccessToken(): string | null {
    return this.userTokensDto?.accessToken || null;
  }

  public getRefreshToken(): string | null {
    return this.userTokensDto?.refreshToken || null;
  }
  // TODO del;
  public setUserDto(userInfo: UserDto | null = null): void {
    this.userInfo = userInfo;
  }
  // TODO del;
  public setUserTokensDto(userTokensDto: UserTokensDto | null = null): void {
    this.userTokensDto = this.setUserTokensDtoToLocalStorage(userTokensDto);
  }
  // TODO del;
  public login(nickname: string, password: string): Promise<LoginUserResponseDto | HttpErrorResponse | undefined> {
    if (!nickname || !password) {
      return Promise.reject();
    }

    this.userTokensDto = this.setUserTokensDtoToLocalStorage(null);
    return this.userApiService.login({ nickname, password }).then((response: LoginUserResponseDto | HttpErrorResponse | undefined) => {
      let userResponseDto: LoginUserResponseDto = response as LoginUserResponseDto;
      this.userInfo = { ...userResponseDto.userDto } as UserDto;
      this.userTokensDto = this.setUserTokensDtoToLocalStorage(userResponseDto.userTokensDto);
      return userResponseDto;
    });
  }
  
  public isCheckRefreshToken(method: string, url: string): boolean {
    return this.userApiService.isCheckRefreshToken(method, url);
  }

  public refreshToken(): Promise<UserTokensDto | HttpErrorResponse> {
    if (!this.userTokensDto?.refreshToken) {
      return Promise.reject();
    }
    return this.userApiService
      .refreshToken({ token: this.userTokensDto.refreshToken })
      .then((response: HttpErrorResponse | UserTokensDto | undefined) => {
        this.userTokensDto = this.setUserTokensDtoToLocalStorage(response as UserTokensDto);
        return response as UserTokensDto;
      })
      .catch((error) => {
        // Remove "Token" values in LocalStorage.
        this.userTokensDto = this.setUserTokensDtoToLocalStorage(null);
        // Return error.
        throw error;
      });
  }

  public logout(): Promise<void | HttpErrorResponse> {
    if (!this.userTokensDto?.accessToken) {
      return Promise.reject();
    }
    return this.userApiService.logout()
      .finally(() => {
        // Reset authorization settings even if an error occurs.
        this.userInfo = null;
        this.userTokensDto = this.setUserTokensDtoToLocalStorage(null);
        return;
      });
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
  // TODO del;
  public async getCurrentUser(): Promise<UserDto | HttpErrorResponse | undefined> {
    const userDto: UserDto = (await this.userApiService.currentUser()) as UserDto;
    this.userInfo = { ...userDto } as UserDto;
    return Promise.resolve(userDto);
  }

  public modifyProfile(
    id: number, modifyProfileDto: ModifyProfileDto, file?: File | null
  ): Promise<UserProfileDto | HttpErrorResponse | undefined> {
    return this.userApiService.modifyProfile(id, modifyProfileDto, file);
  }
  
  public new_password(updatePasswordDto: UpdatePasswordDto): Promise<UserDto | HttpErrorResponse | undefined> {
    return this.userApiService.new_password(updatePasswordDto);
  }

  // ** Private Api **
  // TODO del;
  private updateItemInLocalStorage(name: string, value: string | null): void {
    if (!!name) {
      if (!!value) {
        localStorage.setItem(name, value);
      } else {
        localStorage.removeItem(name);
      }
    }
  }
  // TODO del;
  private setUserTokensDtoToLocalStorage(userTokensDto: UserTokensDto | null): UserTokensDto | null {
    const accessToken = userTokensDto?.accessToken || null;
    this.updateItemInLocalStorage(ACCESS_TOKEN, accessToken);
    const refreshToken = userTokensDto?.refreshToken || null;
    this.updateItemInLocalStorage(REFRESH_TOKEN, refreshToken);
    return !!userTokensDto ? { ...userTokensDto } : null;
  }
  // TODO del;
  private getUserTokensDtoFromLocalStorage(): UserTokensDto | null {
    let result: UserTokensDto | null = null;
    const accessToken = localStorage.getItem(ACCESS_TOKEN);
    const refreshToken = localStorage.getItem(REFRESH_TOKEN);
    if (!!accessToken && !!refreshToken) {
      result = { accessToken, refreshToken };
    }
    return result;
  }
}
