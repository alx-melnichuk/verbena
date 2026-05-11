import { CommonModule } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnInit, ViewEncapsulation } from "@angular/core";
import { TranslateService } from "@ngx-translate/core";
import { LocaleSrv } from "../../common/locale-srv";
import { SessionSrv } from "../../common/session-srv";
import { StringDateTime } from "../../common/string-date-time";
import { ItemView, ItemViewPage } from "../../components/view-list-by-pages/view-list-by-pages";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { StreamsPeriodDto, PageStreamAndTagsDto, StreamDtoUtil } from "../../lib-stream/stream-dto";
import { StreamSrv } from "../../lib-stream/stream-srv";
import { DateUtil } from "../../utils/date.utils";
import { HttpErrorUtil } from "../../utils/http-error.util";
import { StringDateTimeUtil } from "../../utils/string-date-time.util";
import { PanelStreamList } from "../panel-stream-list/panel-stream-list";

const CLND_DELTA_TO_FUTURE = 10;
const CLND_DELTA_TO_PAST = 10;
const STRM_LIMIT_DEF = 10;
const EVNT_LIMIT_DEF = 12;


@Component({
    selector: "app-pg-stream-list",
    exportAs: "appPgStreamList",
    standalone: true,
    imports: [CommonModule, PanelStreamList],
    templateUrl: "./pg-stream-list.html",
    styleUrl: "./pg-stream-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgStreamList implements OnInit {
    private translate: TranslateService = inject(TranslateService);
    private alertSrv: AlertSrv = inject(AlertSrv);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private sessionSrv: SessionSrv = inject(SessionSrv);
    private streamSrv: StreamSrv = inject(StreamSrv);

    // ** "Calendar" **
    public clndIsLoading = false;
    public clndMarkedDates: StreamsPeriodDto[] = [];
    public clndMaxDate: Date;
    public clndMinDate: Date;
    public clndMonth: Date = new Date();

    // ** "Streams" **
    // "Parameters for streams"
    public strmMaxSizeRows: number = STRM_LIMIT_DEF * 3;
    // "Future Streams"
    public strmFtrDeletedId: number | null | undefined;
    public strmFtrIsLoading: boolean = false;
    public strmFtrIsReset: boolean = false;
    public strmFtrItemPage: ItemViewPage | null | undefined;
    // "Past Streams"
    public strmPstDeletedId: number | null | undefined;
    public strmPstIsLoading: boolean = false;
    public strmPstIsReset: boolean = false;
    public strmPstItemPage: ItemViewPage | null | undefined;
    // "Event Streams by Date"
    public evntMaxSizeRows: number = EVNT_LIMIT_DEF * 3;
    public evntDeletedId: number | null | undefined;
    public evntIsLoading: boolean = false;
    public evntIsReset: boolean = false;
    public evntItemPage: ItemViewPage = { list: [], page: 0, limit: 0 };
    public evntDate: Date;

    public userId: number = this.sessionSrv.getUser()?.id || -1;

    private strmSearchDate: Date = new Date();
    private strmFtrPages: number = 0;
    private strmPstPages: number = 0;
    private evntPages: number = 0;

    constructor() {
        const today = new Date();
        const timeZoneOffset = -1 * today.getTimezoneOffset();
        // Get the current day, only the date from 0 hours 0 minutes 0 seconds.
        let now = new Date(today.getFullYear(), today.getMonth(), today.getDate(), 0, timeZoneOffset, 0, 0);
        // Get the minimum date of choice in the calendar (today - 10 years).
        const minDateValue = DateUtil.addYear(now, -CLND_DELTA_TO_PAST);
        this.clndMinDate = DateUtil.addDay(minDateValue, -minDateValue.getDate() + 1);
        // Get the maximum date of choice in the calendar (today + 10 years).
        const maxDateValue = DateUtil.addYear(now, CLND_DELTA_TO_FUTURE);
        const daysInMonth = DateUtil.daysInMonth(maxDateValue);
        this.clndMaxDate = DateUtil.addDay(maxDateValue, daysInMonth - maxDateValue.getDate());
        // Set the calendar month to the current date.
        this.clndMonth = new Date(Date.now());
        // Set the date for the shortlist of streams to the current date.
        this.evntDate = new Date(Date.now());
        this.evntDate.setHours(0, 0, 0, 0);
    }

    ngOnInit(): void {
        this.loadFutureAndPastStreamsAndSchedule();
    }

    // ** Public API **

    // ** "Streams Calendar" panel-stream-calendar **

    public doCalendarForPeriod(calendarStart: Date, userId: number): Promise<void> {
        const arrayOfPromises: Promise<unknown>[] = [];
        // Get a list of events (streams) for a specified date.
        arrayOfPromises.push(this.getCalendarInfoForPeriod(calendarStart, false, userId));

        if (DateUtil.compareYearMonth(calendarStart, this.evntDate) == 0) {
            // Get the first page of short streams for the selected date.
            arrayOfPromises.push(this.doLoadEventDatePage(this.evntDate, 0, -1, userId));
        } else {
            this.evntIsReset = true;
            this.evntItemPage = { list: [], page: 1, limit: EVNT_LIMIT_DEF };
        }

        return Promise.all(arrayOfPromises)
            .then(() => { return })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(errMsg, "pg-stream-list.error_get_streams_for_active_period");
                throw err;
            })
            .finally(() => this.changeDetector.markForCheck());
    }
    // ** "Streams Event" panel-stream-event **
    public doLoadEventDatePage(date: Date | null, page1: number, limit1: number, userId: number): Promise<void> {
        if (limit1 > 1 && page1 > 0 && this.evntPages > 0 && page1 > this.evntPages) {
            return Promise.resolve();
        }
        const dateStart: Date = new Date(date || this.evntDate);
        dateStart.setHours(0, 0, 0, 0);
        this.evntIsReset = page1 < 1;
        const page = page1 > 0 ? page1 : 1;
        const limit = limit1 > 0 ? limit1 : EVNT_LIMIT_DEF;
        this.evntIsLoading = true;
        return this.streamSrv.getStreamsByDate(userId, dateStart, page, limit)
            .then((response: PageStreamAndTagsDto | HttpErrorResponse | undefined) => {
                this.evntDate = dateStart;
                const streams = (response as PageStreamAndTagsDto);
                this.evntItemPage = { list: StreamDtoUtil.createList(streams.list), page: streams.page, limit };
                if (limit1 > 1) {
                    this.evntPages = streams.pages;
                }
            })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(errMsg, "pg-stream-list.error_get_streams_for_selected_day");
                throw err;
            })
            .finally(() => {
                this.evntIsLoading = false;
                this.changeDetector.markForCheck();
            });

    }
    // ** "Future Stream" and "Past Stream" panel-stream-info **
    public doLoadFuturePage(page1: number, limit1: number): Promise<void> {
        if (limit1 > 1 && page1 > 0 && this.strmFtrPages > 0 && page1 > this.strmFtrPages) {
            return Promise.resolve();
        }
        this.strmFtrIsReset = page1 < 1;
        const page = page1 > 0 ? page1 : 1;
        const limit = limit1 > 0 ? limit1 : STRM_LIMIT_DEF;
        this.strmFtrIsLoading = true;
        return this.streamSrv.getFutureStreamsByPage(this.userId, this.strmSearchDate, page, limit)
            .then((response: PageStreamAndTagsDto | HttpErrorResponse | undefined) => {
                const streams = (response as PageStreamAndTagsDto);
                this.strmFtrItemPage = { list: StreamDtoUtil.createList(streams.list), page: streams.page, limit };
                if (limit1 > 1) {
                    this.strmFtrPages = streams.pages;
                }
            })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(errMsg, "pg-stream-list.error_get_future_streams");
                throw err;
            })
            .finally(() => {
                this.strmFtrIsLoading = false;
                this.changeDetector.markForCheck();
            });
    }

    public doLoadPastPage(page1: number, limit1: number): Promise<void> {
        if (limit1 > 1 && page1 > 0 && this.strmPstPages > 0 && page1 > this.strmPstPages) {
            return Promise.resolve();
        }
        this.strmPstIsReset = page1 < 1;
        const page = page1 > 0 ? page1 : 1;
        const limit = limit1 > 0 ? limit1 : STRM_LIMIT_DEF;
        this.strmPstIsLoading = true;
        return this.streamSrv.getPastStreamsByPage(this.userId, this.strmSearchDate, page, limit)
            .then((response: PageStreamAndTagsDto | HttpErrorResponse | undefined) => {
                const streams = (response as PageStreamAndTagsDto);
                this.strmPstItemPage = { list: StreamDtoUtil.createList(streams.list), page: streams.page, limit };
                if (limit1 > 1) {
                    this.strmPstPages = streams.pages;
                }
            })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(errMsg, "pg-stream-list.error_get_past_streams");
                throw err;
            })
            .finally(() => {
                this.strmPstIsLoading = false;
                this.changeDetector.markForCheck();
            });
    }

    public doResetInfo(): void {
        this.loadFutureAndPastStreamsAndSchedule();
    }

    // ** **

    public doActionDuplicate(streamId: number): void {
        this.streamSrv.redirectToStreamCreationPage(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.streamSrv.redirectToStreamEditingPage(streamId);
    }

    public doActionDelete(info: { isFuture: boolean, id: number }): void {
        if (info.isFuture) {
            this.strmFtrIsLoading = true;
        } else {
            this.strmPstIsLoading = true;
        }
        const index = this.getItemById(this.evntItemPage?.list, info.id);
        this.evntIsLoading = index > -1;
        this.deleteDataStream(info.id)
            .then(() => {
                if (info.isFuture) {
                    this.strmFtrDeletedId = info.id;
                } else {
                    this.strmPstDeletedId = info.id;
                }

                if (index > -1) {
                    this.evntDeletedId = info.id;
                }
            })
            .finally(() => {
                if (info.isFuture) {
                    this.strmFtrIsLoading = false;
                } else {
                    this.strmPstIsLoading = false;
                }
                if (this.evntIsLoading) {
                    this.evntIsLoading = false;
                }
            });
    }

    // ** Private API **

    private loadFutureAndPastStreamsAndSchedule(): void {
        const arrayOfPromises: Promise<unknown>[] = [];

        this.strmSearchDate = new Date();

        // Get the first page of "Future Stream".
        arrayOfPromises.push(this.doLoadFuturePage(0, 0));

        // Get the first page of "Past Stream".
        arrayOfPromises.push(this.doLoadPastPage(0, 0));

        const calendarStart = new Date();
        // Get a list of events (streams) for a specified date.
        arrayOfPromises.push(this.doCalendarForPeriod(calendarStart, this.userId));

        this.changeDetector.markForCheck();
        Promise.all(arrayOfPromises)
            .finally(() => {
                this.changeDetector.markForCheck();
            });
    }
    /** Delete a stream by its ID. */
    private deleteDataStream(streamId: number): Promise<void> {
        this.changeDetector.markForCheck();
        // Delete a stream by its ID.
        return this.streamSrv.deleteStream(streamId)
            .then(() => {
                return Promise.resolve();
            })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                const message = this.translate.instant(errMsg);
                const title = this.translate.instant("pg-stream-list.error_delete_stream");
                this.alertSrv.showError(message, title);
                throw err;
            })
            .finally(() => {
                this.changeDetector.markForCheck();
            });
    }
    // ** "Streams Calendar" **
    private getOnlyDate(value: Date | null): string | null {
        return value == null ? null
            : value.getFullYear() + "-" + ("00" + (value.getMonth() + 1)).slice(-2) + "-" + ("00" + value.getDate()).slice(-2);
    }
    private convertStringDateTimeToStreamsPeriodDto(response: StringDateTime[]): StreamsPeriodDto[] {
        const result: StreamsPeriodDto[] = [];
        if (Array.isArray(response)) {
            const obj: { [key: string]: number } = {};
            for (let idx = 0; idx < response.length; idx++) {
                const itemDate: Date | null = StringDateTimeUtil.toDate(response[idx]);
                if (!itemDate) continue;
                itemDate.setHours(0, 0, 0, 0);
                const itemLocal = this.getOnlyDate(itemDate);
                if (!itemLocal) continue;
                obj[itemLocal] = (obj[itemLocal] || 0) + 1;
            }
            const keys = Object.keys(obj);
            for (let i = 0; i < keys.length; i++) {
                result.push({ date: keys[i], count: obj[keys[i]] });
            }
        }
        return result;
    }
    /** Get calendar information for a period. */
    private getCalendarInfoForPeriod(
        startDate: Date, isRequired: boolean, userId: number
    ): Promise<StreamsPeriodDto[] | HttpErrorResponse | undefined> {
        const date: Date = new Date(startDate);
        date.setHours(0, 0, 0, 0);
        const startMonth = DateUtil.dateFirstDayOfMonth(date);
        if (!isRequired && DateUtil.compare(this.clndMonth, startMonth) == 0) {
            return Promise.resolve(undefined);
        }
        this.clndMonth = startMonth;
        const endMonth: Date = DateUtil.dateLastDayOfMonth(startMonth);
        endMonth.setHours(23, 59, 59, 999);
        const start = startMonth.toISOString();
        const finish = endMonth.toISOString();
        this.clndIsLoading = true;
        return this.streamSrv.getCalendarStreamsByDate(userId, start, finish)
            .then((response: StringDateTime[] | HttpErrorResponse | undefined) => {
                this.clndMarkedDates = this.convertStringDateTimeToStreamsPeriodDto(response as StringDateTime[]);
                return this.clndMarkedDates;
            })
            .finally(() => {
                this.clndIsLoading = false;
            });
    }
    private getItemById(list: ItemView[], deletedId: number): number {
        let index = -1;
        for (let n = 0; n < list.length && index == -1; n++) {
            if (deletedId == list[n].id) {
                index = n;
            }
        }
        return index;
    }
}
