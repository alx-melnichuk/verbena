import { CommonModule, KeyValue } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnInit, ViewEncapsulation } from "@angular/core";
import { TranslateService } from "@ngx-translate/core";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { HttpErrorUtil } from "../../utils/http-error.util";
import { PanelStreamList } from "../panel-stream-list/panel-stream-list";
import { StreamSrv } from "../stream-srv";
import { CalendarHandler } from "./calendar-handler";
import { StreamHandler } from "./stream-handler";

const CN_DEFAULT_LIMIT = 7;
const CN_INTERVAL_MINUTES = 5;

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
    private translateService: TranslateService = inject(TranslateService);
    private alertSrv: AlertSrv = inject(AlertSrv);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private streamSrv: StreamSrv = inject(StreamSrv);

    public isRefreshStreamEvent: boolean | null | undefined;

    // "Future Streams"
    public futureStreamHdlr: StreamHandler = new StreamHandler(this.streamSrv, true, CN_DEFAULT_LIMIT, CN_INTERVAL_MINUTES);
    // "Past Streams"
    public pastStreamHdlr: StreamHandler = new StreamHandler(this.streamSrv, false, CN_DEFAULT_LIMIT, CN_INTERVAL_MINUTES);
    // "Calendar"
    public calendarHdlr: CalendarHandler = new CalendarHandler(this.streamSrv);

    ngOnInit(): void {
        this.loadFutureAndPastStreamsAndSchedule();
    }

    // ** Public API **

    // ** "Streams Calendar" panel-stream-calendar **

    public doCalendarForPeriod(calendarStart: Date): void {
        const arrayOfPromises: Promise<unknown>[] = [];
        // Get a list of events (streams) for a specified date.
        arrayOfPromises.push(this.calendarHdlr.getCalendarInfoForPeriod(calendarStart, false));

        if (this.calendarHdlr.isShowEvents(calendarStart)) {
            // Get a list of short streams for the selected date.
            arrayOfPromises.push(this.calendarHdlr.getListEventsForDate(this.calendarHdlr.eventsOfDaySelected, 1));
        } else {
            this.calendarHdlr.clearStreamsEvent();
        }

        Promise.all(arrayOfPromises)
            .catch((err: HttpErrorResponse) => {
                const message = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(message, "pg-stream-list.error_get_streams_for_active_period");
                throw err;
            })
            .finally(() => this.changeDetector.markForCheck());
    }

    // ** "Streams Event" panel-stream-event **

    public doStreamEventsForDate(info: { selectedDate: Date | null, pageNum: number }): void {
        // Get the next page of the list of short streams for the selected date.
        this.calendarHdlr.getListEventsForDate(info.selectedDate, info.pageNum)
            .catch((err: HttpErrorResponse) => {
                const message = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(message, "pg-stream-list.error_get_streams_for_selected_day");
                throw err;
            })
            .finally(() => this.changeDetector.markForCheck());
    }

    // ** "Future Stream" and "Past Stream" panel-stream-info **

    public doSearchNextFutureStream(): void {
        // Checking if the data needs to be updated.
        if (this.futureStreamHdlr.isNeedRefreshData()) {
            this.loadFutureAndPastStreamsAndSchedule();
            return;
        }
        // Get the next page of the "Future Stream".
        this.futureStreamHdlr.searchNextStream()
            .then(() => {
                // Refresh view for "panel-stream-event".
                this.isRefreshStreamEvent = !this.isRefreshStreamEvent ? true : undefined;
            })
            .catch((err: HttpErrorResponse) => {
                const message = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(message, "pg-stream-list.error_get_future_streams");
                throw err;
            })
            .finally(() => this.changeDetector.markForCheck());
    }

    public doSearchNextPastStream(): void {
        // Checking if the data needs to be updated.
        if (this.pastStreamHdlr.isNeedRefreshData()) {
            this.loadFutureAndPastStreamsAndSchedule();
            return;
        }
        // Get the next page of "Past Stream".
        this.pastStreamHdlr.searchNextStream()
            .then(() => {
                // Refresh view for "panel-stream-event".
                this.isRefreshStreamEvent = !this.isRefreshStreamEvent ? true : undefined;
            })
            .catch((err: HttpErrorResponse) => {
                const message = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(message, "pg-stream-list.error_get_past_streams");
                throw err;
            })
            .finally(() => this.changeDetector.markForCheck());
    }

    // ** **

    public doActionDuplicate(streamId: number): void {
        this.streamSrv.redirectToStreamCreationPage(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.streamSrv.redirectToStreamEditingPage(streamId);
    }

    public doActionView(streamId: number): void {
        this.streamSrv.redirectToStreamViewPage(streamId);
    }

    public doActionDelete(streamId: number): void {
        this.deleteDataStream(streamId)
            .then(() => {
                // Update the list of next and past streams.
                this.loadFutureAndPastStreamsAndSchedule();
            });
    }

    // ** Private API **

    private loadFutureAndPastStreamsAndSchedule(): void {
        const arrayOfPromises: Promise<unknown>[] = [];
        const now = new Date();
        now.setHours(0, 0, 0, 0);

        this.futureStreamHdlr.clearStream();
        // Get the next page of the "Future Stream".
        arrayOfPromises.push(this.futureStreamHdlr.searchNextStream());

        this.pastStreamHdlr.clearStream();
        // Get the next page of "Past Stream".
        arrayOfPromises.push(this.pastStreamHdlr.searchNextStream());

        this.calendarHdlr.clearStreamsEvent();
        // Get a list of short streams for the selected date.
        arrayOfPromises.push(this.calendarHdlr.getListEventsForDate(this.calendarHdlr.eventsOfDaySelected || now, 1));

        // Get a list of events (streams) for a specified date.
        const calendarMonth = this.calendarHdlr.calendarMonth || now;
        arrayOfPromises.push(this.calendarHdlr.getCalendarInfoForPeriod(calendarMonth, true));

        this.changeDetector.markForCheck();
        Promise.all(arrayOfPromises)
            .catch((err: HttpErrorResponse) => {
                const message = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(message, "pg-stream-list.error_retrieving_data");
                throw err;
            })
            .finally(() => this.changeDetector.markForCheck());

    }

    private deleteDataStream(streamId: number): Promise<void> {
        this.changeDetector.markForCheck();
        // Delete a stream by its ID.
        return this.futureStreamHdlr.deleteDataStream(streamId)
            .catch((err: HttpErrorResponse) => {
                const msg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                const message = this.translateService.instant(msg);
                const title = this.translateService.instant("pg-stream-list.error_delete_stream");
                this.alertSrv.showError(message, title);
                throw err;
            })
            .finally(() => this.changeDetector.markForCheck());

    }
}
