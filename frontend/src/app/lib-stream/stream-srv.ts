import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from "@angular/common/http";
import { inject, Injectable } from "@angular/core";
import { Router } from "@angular/router";
import { lastValueFrom } from "rxjs";
import { ROUTE_BROWSE_VIEW, ROUTE_STREAM_CREATE, ROUTE_STREAM_EDIT } from "../common/routes";
import { StringDateTime } from "../common/string-date-time";
import { Uri } from "../common/uri";
import { HttpParamsUtil } from "../utils/http-params.util";
import {
    SearchStreamsPeriodDto, SearchStreamDto, StreamListDto, SearchStreamEventDto, StreamEventPageDto, StreamDto, StreamState, StreamDtoUtil, UpdateStreamFileDto
} from "./stream-dto";

@Injectable({
    providedIn: "root",
})
export class StreamSrv {
    private router: Router = inject(Router);
    private http: HttpClient = inject(HttpClient);

    /** Get streams calendar
     * @ streams/calendar/:userId/:month/:year
     * @ type get
     * @ query userId?: number;
     * @ query start: StringDateTime;
     * @ query finish: StringDateTime;
     * @ required start, finish
     * @ access public
     */
    public getStreamsPeriod(search: SearchStreamsPeriodDto): Promise<StringDateTime[] | HttpErrorResponse | undefined> {
        const params: HttpParams = HttpParamsUtil.create(search);
        const url = Uri.appUri(`appApi://streams_period`);
        return lastValueFrom(this.http.get<StringDateTime[] | HttpErrorResponse>(url, { params }));
    }
    /** Get streams
     * @ route streams
     * @ example streams?groupBy=date&userId=385e0469/9/2022&orderColumn=title&orderDirection=desc&live=true
     * @ type get
     * @ query pagination (optional):
     * - userId (only for groupBy "date")
     * - key (keyword by tag or date, the date should be YYYY-MM-DD)
     * - live (false, true)
     * - starttime (none, past, future)
     * - groupBy (none / tag / date, none by default)
     * - page (number, 1 by default)
     * - limit (number, 10 by default)
     * - orderColumn (starttime / title, starttime by default)
     * - orderDirection (asc / desc, asc by default)
     * @ access public
     */
    public getStreams(searchStreamDto: SearchStreamDto): Promise<StreamListDto | HttpErrorResponse | undefined> {
        const params: HttpParams = HttpParamsUtil.create(searchStreamDto);
        const url = Uri.appUri("appApi://streams");
        return lastValueFrom(this.http.get<StreamListDto | HttpErrorResponse>(url, { params }));
    }

    public getStreamsEvent(searchStreamEventDto: SearchStreamEventDto): Promise<StreamEventPageDto | HttpErrorResponse | undefined> {
        const params: HttpParams = HttpParamsUtil.create(searchStreamEventDto);
        const url = Uri.appUri("appApi://streams_events");
        return lastValueFrom(this.http.get<StreamEventPageDto | HttpErrorResponse>(url, { params }));
    }

    /** Get stream
     * @ route streams/:streamId
     * @ type get
     * @ params streamId
     * @ required streamId
     * @ access public
     */
    public getStream(id: number): Promise<StreamDto | HttpErrorResponse | undefined> {
        const url = Uri.appUri(`appApi://streams/${id}`);
        return lastValueFrom(this.http.get<StreamDto | HttpErrorResponse>(url))
            .then((response) => StreamDtoUtil.create(response as StreamDto));
    }

    /** Change state stream
     * @ route streams/toggle/:streamId
     * @ type put
     * @ params streamId
     * @ body state ["preparing" | "started" | "stopped" | "paused"]
     * @ required streamId
     * @ access protected
     */
    public toggleStreamState(streamId: number, state: StreamState): Promise<StreamDto | HttpErrorResponse> {
        if (state === StreamState.waiting) {
            return Promise.reject();
        }
        const url = Uri.appUri(`appApi://streams/toggle/${streamId}`);
        return lastValueFrom(this.http.put<StreamDto | HttpErrorResponse>(url, { state: state }))
            .then((response) => StreamDtoUtil.create(response as StreamDto));
    }

    /** Add stream
     * @ route streams
     * @ type post
     * @ body title, description, starttime, tags (array stringify, 4 max)
     * @ files logo (jpg, png and gif only, 5MB)
     * @ required title, description
     * @ access protected
     */
    public createStream(updateStreamFileDto: UpdateStreamFileDto): Promise<StreamDto | HttpErrorResponse | undefined> {
        const formData: FormData = new FormData();
        formData.set("title", updateStreamFileDto.title || "");
        if (!!updateStreamFileDto.descript) {
            formData.set("descript", updateStreamFileDto.descript);
        }
        if (!!updateStreamFileDto.starttime) {
            formData.set("starttime", updateStreamFileDto.starttime);
        }
        if (!!updateStreamFileDto.source) {
            formData.set("source", updateStreamFileDto.source);
        }
        if (!!updateStreamFileDto.tags) {
            formData.set("tags", JSON.stringify(updateStreamFileDto.tags));
        }
        if (!!updateStreamFileDto.logoFile) {
            formData.set("logofile", updateStreamFileDto.logoFile, updateStreamFileDto.logoFile.name);
        }
        const url = Uri.appUri(`appApi://streams`);
        return lastValueFrom(this.http.post<StreamDto | HttpErrorResponse>(url, formData))
            .then((response) => StreamDtoUtil.create(response as StreamDto));
    }

    /** Update stream
     * @ route streams/:streamId
     * @ type put
     * @ params streamId
     * @ body title, descript, starttime, tags (array stringify, 3 max)
     * @ required streamId
     * @ access protected
     */
    public modifyStream(id: number, updateStreamFileDto: UpdateStreamFileDto): Promise<StreamDto | HttpErrorResponse | undefined> {
        const formData: FormData = new FormData();
        if (updateStreamFileDto.title != null) {
            formData.set("title", updateStreamFileDto.title);
        }
        if (updateStreamFileDto.descript != null) {
            formData.set("descript", updateStreamFileDto.descript);
        }
        if (updateStreamFileDto.logoFile !== undefined) {
            const currFile: File = (updateStreamFileDto.logoFile !== null ? updateStreamFileDto.logoFile : new File([], "file"));
            formData.set("logofile", currFile, currFile.name);
        }
        if (!!updateStreamFileDto.starttime) {
            formData.set("starttime", updateStreamFileDto.starttime);
        }
        if (!!updateStreamFileDto.source) {
            formData.set("source", updateStreamFileDto.source);
        }
        if (!!updateStreamFileDto.tags) {
            formData.set("tags", JSON.stringify(updateStreamFileDto.tags));
        }
        const headers = new HttpHeaders({ "enctype": "multipart/form-data" });
        const url = Uri.appUri(`appApi://streams/${id}`);
        return lastValueFrom(this.http.put<StreamDto | HttpErrorResponse>(url, formData, { headers: headers }))
            .then((response) => StreamDtoUtil.create(response as StreamDto));
    }

    /** Delete stream
     * @ route streams/:streamId
     * @ type delete
     * @ params streamId
     * @ required streamId
     * @ access protected
     */
    public deleteStream(streamId: number): Promise<void | HttpErrorResponse | undefined> {
        const url = Uri.appUri(`appApi://streams/${streamId}`);
        return lastValueFrom(this.http.delete<void | HttpErrorResponse>(url));
    }

    public getLinkForVisitors(streamId: number, isFullPath: boolean): string {
        let prefix = ((isFullPath ? Uri.get("appRoot://") : "") as string);
        if (prefix.slice(-1) === "/") {
            prefix = prefix.slice(0, prefix.length - 1);
        }
        return (!!streamId ? prefix + ROUTE_BROWSE_VIEW + "/" + streamId.toString() : "");
    }

    public redirectToStreamCreationPage(streamId: number): void {
        if (!!streamId) {
            window.setTimeout(() => this.router.navigate([ROUTE_STREAM_CREATE], { queryParams: { id: streamId } }), 0);
        }
    }

    public redirectToStreamEditingPage(streamId: number): void {
        if (!!streamId) {
            window.setTimeout(() => this.router.navigateByUrl(ROUTE_STREAM_EDIT + "/" + streamId), 0);
        }
    }

    public redirectToStreamViewPage(streamId: number): void {
        if (!!streamId) {
            window.setTimeout(() => this.router.navigate([ROUTE_BROWSE_VIEW, streamId]), 0);
        }
    }

}


/*function mockStreamDto(): StreamDto {
    return StreamDtoUtil.create({
        id: 254,
        userId: 18,  // Owner id
        title: "trip 2024 to spain 1 - E.Allen", // Custom title (min: 2, max: 255)
        // Custom description (default: "") (min: 2, max: 2048)
        descript: "Description of a beautiful trip 2024 to spain 1 - E.Allen",
        logo: "/assets/images/trip_spain01.jpg", // Link to stream logo, optional (min: 2, max: 255)
        starttime: new Date("2024-01-01T08:00:00.000Z"), // The stream start time. Required on create
        live: false, // Stream live status, false means inactive
        // Stream live state - waiting, preparing, start, paused, stop (waiting by default)
        state: StreamState.waiting, // ["waiting", "preparing", "started", "paused", "stopped"]
        started: null, // The time the stream began.
        paused: null, // The time the stream began pausing.
        stopped: null, // The time the stream stopped.
        source: "obs",
        tags: ["tourism", "spain"],
        createdAt: new Date("2024-03-08T14:55:44.770Z"),
        updatedAt: new Date("2024-03-08T14:55:44.770Z"),
    });
}*/