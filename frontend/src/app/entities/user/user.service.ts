import { HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
// import { Subject } from 'rxjs';
import { UserApiService } from './user-api.service';
import { CreateUserDto, LoginUserResponseDto, UserDto, UserTokensDto } from './user-dto';

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
    console.log(`UserService();`); // #
    this.userTokensDto = this.getUserTokensDTOFromLocalStorage();
  }

  public login(nickname: string, password: string): Promise<LoginUserResponseDto | HttpErrorResponse | undefined> {
    console.log(`UserService.login("${nickname}", "${password}");`); // #
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

  public registration(nickname: string, email: string, password: string): Promise<null | HttpErrorResponse | undefined> {
    if (!nickname || !email || !password) {
      return Promise.reject();
    }
    return this.userApiService.registration({ nickname, email, password });
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
  private setUserTokensDtoToLocalStorage(authenticationDto: UserTokensDto | null): UserTokensDto | null {
    const accessToken = !!authenticationDto ? authenticationDto.accessToken : null;
    this.updateItemInLocalStorage(ACCESS_TOKEN, accessToken);
    const refreshToken = !!authenticationDto ? authenticationDto.refreshToken : null;
    this.updateItemInLocalStorage(REFRESH_TOKEN, refreshToken);
    return !!authenticationDto ? { ...authenticationDto } : null;
  }

  private getUserTokensDTOFromLocalStorage(): UserTokensDto | null {
    let result: UserTokensDto | null = null;
    const accessToken = localStorage.getItem(ACCESS_TOKEN);
    const refreshToken = localStorage.getItem(REFRESH_TOKEN);
    if (!!accessToken && !!refreshToken) {
      result = { accessToken, refreshToken };
    }
    return result;
  }

  //   private updateSessionDTO(sessionDto: SessionDto | null): void {
  //     this.sessionDto = (!!sessionDto ? { ...sessionDto } : null);
  //     this.innSessionDto.next(this.sessionDto);
  //   }
}
