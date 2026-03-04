import { HttpClient, HttpErrorResponse } from "@angular/common/http";
import { Injectable, inject } from "@angular/core";
import { lastValueFrom } from "rxjs";
import { Uri } from "../common/uri";
import { ProfileConfigDto } from "./profile-config-dto";

@Injectable({
    providedIn: "root",
})
export class ProfileConfigSrv {
    private http: HttpClient = inject(HttpClient);

    public profileConfigDto: ProfileConfigDto | null = null;

    public getConfig(): Promise<ProfileConfigDto | HttpErrorResponse | undefined> {
        if (this.profileConfigDto != null) {
            return Promise.resolve({ ...this.profileConfigDto });
        }
        const url = Uri.appUri("appApi://profiles_config");
        return lastValueFrom(this.http.get<ProfileConfigDto | HttpErrorResponse>(url))
            .then((response: ProfileConfigDto | HttpErrorResponse | undefined) => {
                this.profileConfigDto = response as ProfileConfigDto;
                return { ...this.profileConfigDto };
            });
    }
}
