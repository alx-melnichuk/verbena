import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { AlertService } from 'src/app/lib-dialog/alert.service';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { ROUTE_STREAM_CREATE, ROUTE_STREAM_EDIT } from 'src/app/common/routes';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';
import { StringDateTime } from 'src/app/common/string-date-time';

import { StreamDtoUtil } from '../stream-api.interface';
import { StreamListService } from '../stream-list.service';
import { PanelStreamEventComponent } from '../panel-stream-event/panel-stream-event.component';
import { PanelStreamInfoComponent } from '../panel-stream-info/panel-stream-info.component';
import { PanelStreamCalendarComponent } from '../panel-stream-calendar/panel-stream-calendar.component';
import { StreamService } from '../stream.service';
import { StreamCalendarService } from '../stream-calendar.service';

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
//   private routerChangesSub: Subscription;
  public userId: number;
  
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
    // public scheduleService: ScheduleService,
  ) {
    console.log(`StreamListComponent()`); // #-
    const userDto = this.route.snapshot.data['userDto'];
    this.userId = userDto.id;
    console.log(`StreamListComponent() userId: `, this.userId); // #-
    // this.routerChangesSub = this.router.events
    //   .pipe(filter((event) => event instanceof NavigationEnd))
    //   .subscribe(() => {
    //     this.scheduleService.activeDate = moment();
    //     this.scheduleService.selectedDate = moment().format(MOMENT_ISO8601_DATE);
    //     this.loadFutureAndPastStreamsAndSchedule();
    //     this.changeDetector.markForCheck();
    //   });
    this.loadFutureAndPastStreamsAndSchedule();
  }

  // ** Public API **

  public async doRequestNextPageFuture(): Promise<null | HttpErrorResponse> {
    console.log(`\ndoRequestNextPageFuture() 01`); // #
    await this.streamListService.searchNextFutureStream(this.userId);
    this.changeDetector.markForCheck();
    return null;
  }

  public async doRequestNextPagePast(): Promise<null | HttpErrorResponse> {
    console.log(`\ndoRequestNextPagePast() 01`); // #
    await this.streamListService.searchNextPastStream(this.userId);
    this.changeDetector.markForCheck();
    return null;
  }

  public isFuture(startTime: StringDateTime | null): boolean | null {
    return StreamDtoUtil.isFuture(startTime);
  }

  public doRedirectToStreamView(streamId: number): void {
    if (!!streamId) {
      this.streamService.redirectToStreamViewPage(streamId);
    }
  }

  // "Panel Calendar and Events"

  public doChangeSelectedDate(selectedDate: Date | null) {

  }
  //   (requestNextPage)="doChangeSelectedDate()"
  public async doRequestNextPageShortStreams(selectedDate?: StringDateTime): Promise<void> {
    const starttime: StringDateTime = selectedDate || this.streamCalendarService.streamsEventStarttime;
    // Get a list of days for a new calendar period.
    await this.streamCalendarService.getShortStreamsForDate(starttime)
    .finally(() => {
      this.changeDetector.markForCheck();
    });
  }

  public async doChangeCalendar(calendarStart: Date): Promise<void> {
    // Get a list of events (streams) for a specified date.
    await this.streamCalendarService.getStreamsCalendarForDate(calendarStart)
    .finally(() => {
        this.changeDetector.markForCheck();
    });
  }
  /*public async doChangeActiveDate(activeDateStr: StringDate): Promise<null | HttpErrorResponse> {
    if (!activeDateStr) {
      return null;
    }
    const activeDate: moment.Moment = moment(activeDateStr, MOMENT_ISO8601_DATE);
    const userId = (this.userId as string);
    await this.scheduleService.setActivePeriod(userId, activeDate)
      .finally(() => {
        this.changeDetector.markForCheck();
      });
    return null;
  }*/

  private loadFutureAndPastStreamsAndSchedule(): void {
    this.streamListService.clearFutureStream();
    this.doRequestNextPageFuture();

    this.streamListService.clearPastStream();
    this.doRequestNextPagePast();

    const now = new Date();
    now.setHours(0, 0, 0, 0);
    // console.log(`loadFutureAndPastStreamsAndSchedule()`); // #
    // console.log(`now    :`, now); // #
    // console.log(`now.iso:`, now.toISOString()); // #

    this.streamCalendarService.getStreamsCalendarForDate(now);
    // this.doChangeActiveDate(this.scheduleService.activeDate.format(MOMENT_ISO8601_DATE));

    this.doChangeSelectedDate(now);
    // this.doRequestNextPageShortStreams();
  }

  // "Streams"

  public doActionDuplicate(streamId: number): void {
    this.alertService.hide();
    if (!!streamId) {
      this.router.navigate([ROUTE_STREAM_CREATE], { queryParams: { id: streamId } });
    }
  }

  public doActionEdit(streamId: number): void {
    this.alertService.hide();
    if (!!streamId) {
      this.router.navigateByUrl(ROUTE_STREAM_EDIT + '/' + streamId);
    }
  }

  public doActionDelete(info: { id: number, title: string }): void {
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


  // "Streams"

  private async deleteDataStream(streamId: number): Promise<void> {
    this.alertService.hide();
    if (!streamId) {
      return Promise.reject();
    }
    let isRefres = false;
    this.streamService.deleteStream(streamId + 1000)
      .then((response: void | HttpErrorResponse | undefined) => {
        isRefres = true;
      })
      .catch((error: HttpErrorResponse) => {
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'stream_list.error_delete_stream');
        throw error;
      })
      .finally(() => {
        this.changeDetector.markForCheck();
        if (isRefres) {
          Promise.resolve().then(() => {
            this.loadFutureAndPastStreamsAndSchedule();
          });
        }
      });
  }
}
