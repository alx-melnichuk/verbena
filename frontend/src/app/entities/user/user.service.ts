import { HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
// import { Subject } from 'rxjs';
import { UserApiService } from './user-api.service';
import { LoginUserResponseDto, UserDto, UserTokensDto } from './user-dto';

export const ACCESS_TOKEN = 'accessToken';
export const REFRESH_TOKEN = 'refreshToken';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  //   public sessionDto: SessionDto | null = null;
  public userInfo: UserDto | null = null;
  public userTokensDto: UserTokensDto | null = null;
  //   private innSessionDto: Subject<SessionDto | null> = new Subject<SessionDto | null>();
  //   public sessionDTO$ = this.innSessionDto.asObservable();

  constructor(private userApiService: UserApiService) {
    console.log(`#1-UserService();`); // #
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
  
  public isCeckRefreshToken(method: string, url: string): boolean {
    return this.userApiService.isCeckRefreshToken(method, url);
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
      .then(() => {
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

  public async getCurrentUser(): Promise<UserDto | HttpErrorResponse | undefined> {
    const userDto: UserDto = (await this.userApiService.currentUser()) as UserDto;
    this.userInfo = { ...userDto } as UserDto;
    return Promise.resolve(userDto);
  }

  // ** Private **

  private updateItemInLocalStorage(name: string, value: string | null): void {
    if (!!name) {
      if (!!value) {
        localStorage.setItem(name, value);
      } else {
        localStorage.removeItem(name);
      }
    }
  }
  private setUserTokensDtoToLocalStorage(userTokensDto: UserTokensDto | null): UserTokensDto | null {
    const accessToken = userTokensDto?.accessToken || null;
    this.updateItemInLocalStorage(ACCESS_TOKEN, accessToken);
    const refreshToken = userTokensDto?.refreshToken || null;
    this.updateItemInLocalStorage(REFRESH_TOKEN, refreshToken);
    return !!userTokensDto ? { ...userTokensDto } : null;
  }

  private getUserTokensDtoFromLocalStorage(): UserTokensDto | null {
    let result: UserTokensDto | null = null;
    const accessToken = localStorage.getItem(ACCESS_TOKEN);
    const refreshToken = localStorage.getItem(REFRESH_TOKEN);
    if (!!accessToken && !!refreshToken) {
      result = { accessToken, refreshToken };
    }
    return result;
  }

  // ** Private Api **

  //   private updateSessionDTO(sessionDto: SessionDto | null): void {
  //     this.sessionDto = (!!sessionDto ? { ...sessionDto } : null);
  //     this.innSessionDto.next(this.sessionDto);
  //   }
}
