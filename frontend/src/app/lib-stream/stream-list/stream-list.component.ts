import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { PIPE_DATE } from 'src/app/common/constants';
import { ROUTE_STREAM_CREATE, ROUTE_STREAM_EDIT } from 'src/app/common/routes';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

import { PanelStreamCalendarComponent } from '../panel-stream-calendar/panel-stream-calendar.component';
import { PanelStreamEventComponent } from '../panel-stream-event/panel-stream-event.component';
import { PanelStreamInfoComponent } from '../panel-stream-info/panel-stream-info.component';
import { StreamCalendarService } from '../stream-calendar.service';
import { StreamListService } from '../stream-list.service';
import { StreamService } from '../stream.service';

@Component({
  selector: 'app-stream-list',
  standalone: true,
  imports: [CommonModule, TranslateModule, SpinnerComponent, PanelStreamEventComponent, PanelStreamInfoComponent,
    PanelStreamCalendarComponent],
  templateUrl: './stream-list.component.html',
  styleUrls: ['./stream-list.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class StreamListComponent {
  public userId: number;

  readonly formatDate = PIPE_DATE;

  constructor(
    private changeDetector: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router,
    private translateService: TranslateService,
    private dialogService: DialogService,
    private streamService: StreamService,
    private alertService: AlertService,
    public streamListService: StreamListService,
    public streamCalendarService: StreamCalendarService,
  ) {
    const userDto = this.route.snapshot.data['userDto'];
    this.userId = userDto.id;
    this.loadFutureAndPastStreamsAndSchedule();
  }

  // ** Public API **

  // ** "Streams Calendar" panel-stream-calendar **

  public async getCalendarInfoForPeriod(calendarStart: Date): Promise<void> {
    const buffPromise: Promise<unknown>[] = [];
    // Get a list of events (streams) for a specified date.
    buffPromise.push(this.streamCalendarService.getCalendarInfoForPeriod(calendarStart, false));
    if (this.streamCalendarService.isShowEvents(calendarStart)) {
      buffPromise.push(this.getListEventsForDate(this.streamCalendarService.eventsOfDaySelected, 1));
    } else {
      this.streamCalendarService.clearStreamsEvent();
    }
    Promise.all(buffPromise).finally(() => this.changeDetector.markForCheck());
  }

  // ** "Streams Event" panel-stream-event **

  public async getListEventsForDate(selectedDate: Date | null, pageNum: number): Promise<void> {
    // Get the next page of the list of short streams for the selected date.
    await this.streamCalendarService.getListEventsForDate(selectedDate, pageNum)
    .finally(() => {
      this.changeDetector.markForCheck();
    });
  }

  // ** "Future Stream" and "Past Stream" panel-stream-info **

  public async searchNextFutureStream(): Promise<null | HttpErrorResponse> {
    // Get the next page of the "Future Stream".
    await this.streamListService.searchNextFutureStream();
    this.changeDetector.markForCheck();
    return null;
  }

  public async searchNextPastStream(): Promise<null | HttpErrorResponse> {
    // Get the next page of "Past Stream".
    await this.streamListService.searchNextPastStream();
    this.changeDetector.markForCheck();
    return null;
  }

  // ** **

  public redirectToStreamView(streamId: number): void {
    if (!!streamId) {
      this.streamService.redirectToStreamViewPage(streamId);
    }
  }
  public actionDuplicate(streamId: number): void {
    this.alertService.hide();
    if (!!streamId) {
      this.router.navigate([ROUTE_STREAM_CREATE], { queryParams: { id: streamId } });
    }
  }
  public actionEdit(streamId: number): void {
    this.alertService.hide();
    if (!!streamId) {
      this.router.navigateByUrl(ROUTE_STREAM_EDIT + '/' + streamId);
    }
  }
  public actionDelete(info: { id: number, title: string }): void {
    this.alertService.hide();
    if (!!info && !!info.id) {
      this.alertService.hide();
      const message = this.translateService.instant('stream_list.sure_you_want_delete_stream', { title: info.title });
      this.dialogService.openConfirmation(message, '', 'buttons.no', 'buttons.yes')
        .then((result) => {
          if (!!result) {
            this.deleteDataStream(info.id);
          }
        });
    }
  }

  // ** Private API **

  private loadFutureAndPastStreamsAndSchedule(): void {
    // Clear array of "Future Stream"
    this.streamListService.clearFutureStream();
    // Clear array of "Past Stream"
    this.streamListService.clearPastStream();

    const buffPromise: Promise<unknown>[] = [];
    // Get the next page of the "Future Stream".
    buffPromise.push(this.streamListService.searchNextFutureStream());
    // Get the next page of "Past Stream".
    buffPromise.push(this.streamListService.searchNextPastStream());
    const now = new Date();
    now.setHours(0, 0, 0, 0);
    const selectedDate = this.streamCalendarService.eventsOfDaySelected || now;
    // Get a list of short streams for the selected date.
    buffPromise.push(this.streamCalendarService.getListEventsForDate(selectedDate, 1));
    const selectedMonth = this.streamCalendarService.calendarMonth || now;
    // Get a list of events (streams) for a specified date.
    buffPromise.push(this.streamCalendarService.getCalendarInfoForPeriod(selectedMonth, true));

    Promise.all(buffPromise).finally(() => this.changeDetector.markForCheck());
  }

  private async deleteDataStream(streamId: number): Promise<void> {
    this.alertService.hide();
    if (!streamId) {
      return Promise.reject();
    }
    let isRefres = false;
    this.streamService.deleteStream(streamId)
      .then(() => isRefres = true)
      .catch((error: HttpErrorResponse) => {
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_delete_stream');
        throw error;
      })
      .finally(() => {
        this.changeDetector.markForCheck();
        if (isRefres) {
          Promise.resolve().then(() => this.loadFutureAndPastStreamsAndSchedule());
        }
      });
  }
}
