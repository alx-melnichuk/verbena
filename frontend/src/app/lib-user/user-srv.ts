import { HttpClient, HttpErrorResponse, HttpParams } from "@angular/common/http";
import { Injectable, inject } from "@angular/core";
import { lastValueFrom } from "rxjs";
import { environment } from "../../environments/environment";
import { Uri } from "../common/uri";
import { HttpParamsUtil } from "../utils/http-params.util";
import {
    RegistrUserDto, RecoveryUserDto, LoginResponseDto, LoginDto, UserDtoUtil, UserTokenResponseDto, UserTokenDto, UniquenessDto, UserDto, UserShortDto, UserShortDtoUtil, TokenUpdate
} from "./user-dto";

@Injectable({
    providedIn: 'root',
})
export class UserSrv implements TokenUpdate {
    private http: HttpClient = inject(HttpClient);

    constructor() {
        if (environment.logLevel > 0) { console.log(`UserSrv(); // 3 service`); }
    }

    // ** Public Api **

    public registration(nickname: string, email: string, password: string): Promise<null | HttpErrorResponse | undefined> {
        if (!nickname || !email || !password) {
            return Promise.reject();
        }
        const url = Uri.appUri("appApi://registration");
        const registrUserDto: RegistrUserDto = { nickname, email, password };
        return lastValueFrom(this.http.post<null | HttpErrorResponse>(url, registrUserDto));
    }

    public recovery(email: string): Promise<null | HttpErrorResponse | undefined> {
        if (!email) {
            return Promise.reject();
        }
        const url = Uri.appUri("appApi://recovery");
        const recoveryProfileDto: RecoveryUserDto = { email };
        return lastValueFrom(this.http.post<null | HttpErrorResponse>(url, recoveryProfileDto));
    }

    public login(nickname: string, password: string): Promise<LoginResponseDto | HttpErrorResponse | undefined> {
        if (!nickname || !password) {
            return Promise.reject();
        }
        const url = Uri.appUri("appApi://login");
        const loginProfileDto: LoginDto = { nickname, password };
        return lastValueFrom(this.http.post<LoginResponseDto | HttpErrorResponse>(url, loginProfileDto));
    }

    public logout(): Promise<void | HttpErrorResponse> {
        const url = Uri.appUri("appApi://logout");
        return lastValueFrom(this.http.post<void | HttpErrorResponse>(url, null));
    }

    public refreshToken(refreshToken: string): Promise<UserTokenResponseDto | HttpErrorResponse> {
        const url = Uri.appUri("appApi://token");
        const userTokenDto: UserTokenDto = { token: refreshToken };
        return lastValueFrom(this.http.post<UserTokenResponseDto | HttpErrorResponse>(url, userTokenDto));
    }

    // ** Uniqueness of "nickname" and "email". **

    public uniqueness(nickname: string, email: string): Promise<UniquenessDto | HttpErrorResponse | undefined> {
        if (!nickname && !email) {
            return Promise.resolve(undefined);
        }
        const params: HttpParams = HttpParamsUtil.create({ nickname: (!nickname ? null : nickname), email: (!email ? null : email) });

        const url = Uri.appUri("appApi://users_uniqueness");
        return lastValueFrom(this.http.get<UniquenessDto | HttpErrorResponse>(url, { params }));
    }

    public getCurrentUser(): Promise<UserDto | HttpErrorResponse | undefined> {
        const url = Uri.appUri("appApi://profiles_current");
        return lastValueFrom(this.http.get<UserDto | HttpErrorResponse>(url));
    }

    public getUserShort(userId: number): Promise<UserShortDto | HttpErrorResponse | undefined> {
        if (!userId) {
            return Promise.reject();
        }
        const url = Uri.appUri(`appApi://profiles_mini/${userId}`);
        return lastValueFrom(this.http.get<UserShortDto | HttpErrorResponse>(url))
            .then((response: UserShortDto | HttpErrorResponse | undefined) => {
                return UserShortDtoUtil.new(response as UserShortDto)
            });
    }

    // ** Private Api **

}
