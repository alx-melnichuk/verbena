import { HttpClient, HttpErrorResponse, HttpHeaders } from "@angular/common/http";
import { inject, Injectable } from "@angular/core";
import { lastValueFrom } from "rxjs";
import { Uri } from "../common/uri";
import { ModifyProfileDto, NewPasswordProfileDto, ProfileDto } from "./profile-dto";

@Injectable({
    providedIn: "root",
})
export class ProfileSrv {
    private http: HttpClient = inject(HttpClient);

    public async modifyProfile(modifyProfileDto: ModifyProfileDto, file?: File | null): Promise<ProfileDto | HttpErrorResponse | undefined> {
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
        if (modifyProfileDto.settings != null && typeof modifyProfileDto.settings === "object") {
            formData.set("settings", JSON.stringify(modifyProfileDto.settings));
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
            return lastValueFrom(this.http.put<ProfileDto | HttpErrorResponse>(url, formData, { headers: headers }));
        }
    }

    public newPassword(newPasswordProfileDto: NewPasswordProfileDto): Promise<ProfileDto | HttpErrorResponse | undefined> {
        if (!newPasswordProfileDto.password && !newPasswordProfileDto.newPassword) {
            return Promise.resolve(undefined);
        }
        const url = Uri.appUri("appApi://profiles_new_password");
        return lastValueFrom(this.http.put<ProfileDto | HttpErrorResponse>(url, newPasswordProfileDto));
    }

    public deleteCurrentProfile(): Promise<ProfileDto | HttpErrorResponse | undefined> {
        const url = Uri.appUri("appApi://profiles_current");
        return lastValueFrom(this.http.delete<ProfileDto | HttpErrorResponse>(url));
    }

    public getCurrentProfile(): Promise<ProfileDto | HttpErrorResponse | undefined> {
        const url = Uri.appUri("appApi://profiles_current");
        return lastValueFrom(this.http.get<ProfileDto | HttpErrorResponse>(url));
    }
}
