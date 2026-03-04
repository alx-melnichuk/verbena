import { HttpClient, HttpErrorResponse } from "@angular/common/http";
import { Injectable, inject } from "@angular/core";
import { lastValueFrom } from "rxjs";
import { Uri } from "../common/uri";
import { StreamConfigDto } from "./stream-config-dto";

@Injectable({
    providedIn: "root",
})
export class StreamConfigSrv {
    private http: HttpClient = inject(HttpClient);

    public streamConfigDto: StreamConfigDto | null = null;

    public getConfig(): Promise<StreamConfigDto | HttpErrorResponse | undefined> {
        if (this.streamConfigDto != null) {
            return Promise.resolve({ ...this.streamConfigDto });
        }
        const url = Uri.appUri("appApi://streams_config");
        return lastValueFrom(this.http.get<StreamConfigDto | HttpErrorResponse>(url))
            .then((response: StreamConfigDto | HttpErrorResponse | undefined) => {
                this.streamConfigDto = response as StreamConfigDto;
                return { ...this.streamConfigDto };
            });
    }
}
