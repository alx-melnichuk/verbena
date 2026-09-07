import { HttpClient, HttpErrorResponse, HttpHeaders } from "@angular/common/http";
import { inject, Injectable } from "@angular/core";
import { lastValueFrom } from "rxjs";
import { Uri } from "../common/uri";
import { SettingsDto, UserDto } from "../lib-user/user-dto";
import { ModifyProfileDto, NewPasswordProfileDto } from "./profile-dto";

@Injectable({
    providedIn: "root",
})
export class ProfileSrv {
    private http: HttpClient = inject(HttpClient);

    public async modifyProfile(modifyProfileDto: ModifyProfileDto, file?: File | null): Promise<UserDto | HttpErrorResponse | undefined> {
        const formData: FormData = new FormData();
        if (modifyProfileDto.nickname != null) {
            formData.set("nickname", modifyProfileDto.nickname);
        }
        if (modifyProfileDto.email != null) {
            formData.set("email", modifyProfileDto.email);
        }
        if (modifyProfileDto.role != null) {
            formData.set("role", modifyProfileDto.role);
        }
        if (modifyProfileDto.descript != null) {
            formData.set("descript", modifyProfileDto.descript);
        }
        if (modifyProfileDto.theme != null) {
            formData.set("theme", modifyProfileDto.theme);
        }
        if (modifyProfileDto.locale != null) {
            formData.set("locale", modifyProfileDto.locale);
        }
        if (file !== undefined) {
            const currFile: File = (file !== null ? file : new File([], "file"));
            formData.set("avatarfile", currFile, currFile.name);
        }
        let cnt = 0;
        for (const _key of formData.keys()) { cnt++; }
        if (cnt == 0) {
            return Promise.resolve(undefined);
        } else {
            const headers = new HttpHeaders({ "enctype": "multipart/form-data" });
            const url = Uri.appUri(`appApi://profiles`);
            return lastValueFrom(this.http.put<UserDto | HttpErrorResponse>(url, formData, { headers: headers }));
        }
    }

    public async modifySettings(settingsDto: SettingsDto | null | undefined): Promise<UserDto | HttpErrorResponse | undefined> {
        let value = undefined;
        if (settingsDto === null) {
            value = "{}"; // Reset
        } else if (settingsDto !== undefined) {
            const s1 = JSON.stringify(settingsDto);
            value = (s1 != "{}" ? s1 : value);
        }
        if (!value) {
            return Promise.resolve(undefined);
        }
        const formData: FormData = new FormData();
        formData.set("settings", value);

        const headers = new HttpHeaders({ "enctype": "multipart/form-data" });
        const url = Uri.appUri(`appApi://profiles`);
        return lastValueFrom(this.http.put<UserDto | HttpErrorResponse>(url, formData, { headers: headers }));
    }

    public newPassword(newPasswordProfileDto: NewPasswordProfileDto): Promise<UserDto | HttpErrorResponse | undefined> {
        if (!newPasswordProfileDto.password && !newPasswordProfileDto.newPassword) {
            return Promise.resolve(undefined);
        }
        const url = Uri.appUri("appApi://profiles_new_password");
        return lastValueFrom(this.http.put<UserDto | HttpErrorResponse>(url, newPasswordProfileDto));
    }

    public deleteCurrentProfile(): Promise<UserDto | HttpErrorResponse | undefined> {
        const url = Uri.appUri("appApi://profiles_current");
        return lastValueFrom(this.http.delete<UserDto | HttpErrorResponse>(url));
    }
}
