import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';

import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';
import { CN_DELETE_STREAM, StreamListComponent } from 'src/app/lib-stream/stream-list/stream-list.component';
import { StreamService } from 'src/app/lib-stream/stream.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

import { CalendarHandler } from './calendar-handler';
import { StreamHandler } from './stream-handler';

const CN_DEFAULT_LIMIT = 7;
const CN_INTERVAL_MINUTES = 5;

@Component({
    selector: 'app-pg-stream-list',
    standalone: true,
    imports: [CommonModule, StreamListComponent],
    templateUrl: './pg-stream-list.component.html',
    styleUrl: './pg-stream-list.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgStreamListComponent implements OnInit {
    private alertService: AlertService = inject(AlertService);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private streamService: StreamService = inject(StreamService);

    public errObj: KeyValue<string, string> | null = null;
    public isRefreshStreamEvent: boolean | null | undefined;
    public profileDto: ProfileDto | null = this.route.snapshot.data['profileDto'];

    // "Future Streams"
    public futureStreamHdlr: StreamHandler = new StreamHandler(this.streamService, true, CN_DEFAULT_LIMIT, CN_INTERVAL_MINUTES);
    // "Past Streams"
    public pastStreamHdlr: StreamHandler = new StreamHandler(this.streamService, false, CN_DEFAULT_LIMIT, CN_INTERVAL_MINUTES);
    // "Calendar"
    public calendarHdlr: CalendarHandler = new CalendarHandler(this.streamService);

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
            .catch((error) => {
                this.alertService.showError(
                    HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], 'stream_list.error_get_streams_for_active_period');
                throw error;
            })
            .finally(() => this.changeDetector.markForCheck());
    }

    // ** "Streams Event" panel-stream-event **

    public doStreamEventsForDate(info: { selectedDate: Date | null, pageNum: number }): void {
        // Get the next page of the list of short streams for the selected date.
        this.calendarHdlr.getListEventsForDate(info.selectedDate, info.pageNum)
            .catch((error) => {
                this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0]
                    , 'stream_list.error_get_streams_for_selected_day');
                throw error;
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
            .catch((error) => {
                this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], 'stream_list.error_get_future_streams');
                throw error;
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
            .catch((error) => {
                this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], 'stream_list.error_get_past_streams');
                throw error;
            })
            .finally(() => this.changeDetector.markForCheck());
    }

    // ** **

    public doActionDuplicate(streamId: number): void {
        this.streamService.redirectToStreamCreationPage(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.streamService.redirectToStreamEditingPage(streamId);
    }

    public doActionView(streamId: number): void {
        this.streamService.redirectToStreamViewPage(streamId);
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
            .catch((error) => {
                this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], 'stream_list.error_retrieving_data');
                throw error;
            })
            .finally(() => this.changeDetector.markForCheck());

    }

    private deleteDataStream(streamId: number): Promise<void> {
        this.errObj = null;
        this.changeDetector.markForCheck();
        // Delete a stream by its ID.
        return this.futureStreamHdlr.deleteDataStream(streamId)
            .catch((error) => {
                if (error instanceof HttpErrorResponse) {
                    const value = HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0];
                    this.errObj = { "key": CN_DELETE_STREAM, value };
                }
                throw error;
            })
            .finally(() => this.changeDetector.markForCheck());

    }
}
