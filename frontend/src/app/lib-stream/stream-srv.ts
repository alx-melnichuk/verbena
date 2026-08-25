import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from "@angular/common/http";
import { inject, Injectable } from "@angular/core";
import { Router } from "@angular/router";
import { lastValueFrom } from "rxjs";
import { ROUTE_BROWSE_VIEW, ROUTE_STREAM_CREATE, ROUTE_STREAM_EDIT } from "../common/routes";
import { StringDateTime } from "../common/string-date-time";
import { Uri } from "../common/uri";
import { HttpParamsUtil } from "../utils/http-params.util";
import {
    SearchStreamAndTagsDto, StreamDto, StreamDtoUtil, PageStreamAndTagsDto, StreamState, UpdateStreamFileDto,
    SearchStreamTagDto, PageStreamTagDto
} from "./stream-dto";

@Injectable({
    providedIn: "root",
})
export class StreamSrv {
    private router: Router = inject(Router);
    private http: HttpClient = inject(HttpClient);

    /** Get streams popular tags
     * @ route streams_popular_tags
     * @ type get
     * @ access public
     */
    public getStreamsPopularTags(search: SearchStreamTagDto): Promise<PageStreamTagDto | HttpErrorResponse> {
        const params: HttpParams = HttpParamsUtil.create(search);
        const url = Uri.appUri("appApi://streams_popural_tags");
        return lastValueFrom(this.http.get<PageStreamTagDto | HttpErrorResponse>(url, { params }));
    }
    /** Get streams calendar
     * @ streams/calendar/:userId/:month/:year
     * @ type get
     * @ query userId?: number;
     * @ query start: StringDateTime;
     * @ query finish: StringDateTime;
     * @ required start, finish
     * @ access public
     */
    public getCalendarStreamsByDate(
        userId: number, start: StringDateTime, finish: StringDateTime
    ): Promise<StringDateTime[] | HttpErrorResponse | undefined> {
        const params: HttpParams = HttpParamsUtil.create({ userId, start, finish });
        const url = Uri.appUri("appApi://streams_calendar");
        return lastValueFrom(this.http.get<StringDateTime[] | HttpErrorResponse>(url, { params }));
    }

    public getStreamsByDate(
        userId: number, date: Date, page: number, limit: number
    ): Promise<PageStreamAndTagsDto | HttpErrorResponse> {
        const startDate: Date = new Date(date);
        startDate.setHours(0, 0, 0, 0);
        const finishDate: Date = new Date(startDate);
        finishDate.setHours(23, 59, 59, 999);

        const searchStreamDto: SearchStreamAndTagsDto = {
            userId,
            filter: "period",
            starttime: startDate.toISOString(),
            finishtime: finishDate.toISOString(),
            page: (page != null && page > 0 ? page : 1), // default = 1;
            limit: (limit != null && limit > 0 ? limit : 10), // Min(1) Max(100)        
        };
        const params: HttpParams = HttpParamsUtil.create(searchStreamDto);
        const url = Uri.appUri("appApi://streams");
        return lastValueFrom(this.http.get<PageStreamAndTagsDto | HttpErrorResponse>(url, { params }));
    }

    public getFutureStreamsByPage(
        userId: number, starttime: Date, page: number, limit: number
    ): Promise<PageStreamAndTagsDto | HttpErrorResponse> {
        const searchStreamDto: SearchStreamAndTagsDto = {
            userId,
            filter: "future",
            starttime: starttime.toISOString(),
            page: (page != null && page > 0 ? page : 1), // default = 1;
            limit: (limit != null && limit > 0 ? limit : 10), // Min(1) Max(100)        
        };
        const params: HttpParams = HttpParamsUtil.create(searchStreamDto);
        const url = Uri.appUri("appApi://streams");
        return lastValueFrom(this.http.get<PageStreamAndTagsDto | HttpErrorResponse>(url, { params }));
    }

    public getPastStreamsByPage(
        userId: number, starttime: Date, page: number, limit: number
    ): Promise<PageStreamAndTagsDto | HttpErrorResponse> {
        const searchStreamDto: SearchStreamAndTagsDto = {
            userId,
            filter: "past",
            starttime: starttime.toISOString(),
            sortDesc: true,
            page: (page != null && page > 0 ? page : 1), // default = 1;
            limit: (limit != null && limit > 0 ? limit : 10), // Min(1) Max(100)        
        };
        const params: HttpParams = HttpParamsUtil.create(searchStreamDto);
        const url = Uri.appUri("appApi://streams");
        return lastValueFrom(this.http.get<PageStreamAndTagsDto | HttpErrorResponse>(url, { params }));
    }

    public getStreamsWithTagByPage(
        tag: string | null, live: boolean | undefined, page: number, limit: number
    ): Promise<PageStreamAndTagsDto | HttpErrorResponse> {
        const searchStreamDto: SearchStreamAndTagsDto = {
            page: (page != null && page > 0 ? page : 1), // default = 1;
            limit: (limit != null && limit > 0 ? limit : 10), // Min(1) Max(100)        
        };
        if (!!tag) {
            searchStreamDto.tag = tag;
        }
        if (live != undefined) {
            searchStreamDto.live = live;
        }
        const params: HttpParams = HttpParamsUtil.create(searchStreamDto);
        const url = Uri.appUri("appApi://streams");
        return lastValueFrom(this.http.get<PageStreamAndTagsDto | HttpErrorResponse>(url, { params }));
    }
    /** Get streams
     * @ route streams
     * @ type get
     * @ query pagination (optional):
     * - userId
     * - live (false, true)
     * - filter ("future", "past", "start")
     * - starttime (StringDateTime)
     * - sortDesc  (false, true)
     * - tag (string)
     * - page (number, 1 by default)
     * - limit (number, 10 by default)
     * @ access public
     */
    public getStreams(searchStreamDto: SearchStreamAndTagsDto): Promise<PageStreamAndTagsDto | HttpErrorResponse | undefined> {
        const params: HttpParams = HttpParamsUtil.create(searchStreamDto);
        const url = Uri.appUri("appApi://streams");
        return lastValueFrom(this.http.get<PageStreamAndTagsDto | HttpErrorResponse>(url, { params }));
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
        const url = Uri.appUri("appApi://streams");
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
    public deleteStream(streamId: number): Promise<StreamDto | HttpErrorResponse | undefined> {
        const url = Uri.appUri(`appApi://streams/${streamId}`);
        return lastValueFrom(this.http.delete<StreamDto | HttpErrorResponse>(url))
            .then((response) => StreamDtoUtil.create(response as StreamDto));
    }

    public getLinkToStream(streamId: number, isFullPath: boolean): string {
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
