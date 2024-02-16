import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';

import { PageData, PageDataUtil } from 'src/app/common/page-data';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { ROUTE_STREAM_CREATE, ROUTE_STREAM_EDIT } from 'src/app/common/routes';
import { AlertService } from 'src/app/lib-dialog/alert.service';

import { StringDateTime, StreamDtoUtil, StreamDto } from '../stream-api.interface';
import { StreamListService } from '../stream-list.service';
import { PanelStreamMiniComponent } from '../panel-stream-mini/panel-stream-mini.component';

@Component({
  selector: 'app-stream-list',
  standalone: true,
  imports: [CommonModule, TranslateModule, SpinnerComponent, PanelStreamMiniComponent],
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
    // private translateService: TranslateService,
    // private dialogService: DialogService,
    // private streamService: StreamService,
    private alertService: AlertService,
    public streamListService: StreamListService,
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

  public async doRequestNextPageFuture(nextPageData: PageData): Promise<null | HttpErrorResponse> {
    if (!this.streamListService.futureStreamsDto) {
      return null;
    }
    const pageData = PageDataUtil.create(this.streamListService.futureStreamsDto);
    if (!!pageData && !!nextPageData && PageDataUtil.checkNextPage(pageData, nextPageData)) {
      await this.streamListService.getFutureStream(this.userId, nextPageData);
      this.changeDetector.markForCheck();
    }
    return null;
  }

  public async doRequestNextPagePast(nextPageData: PageData): Promise<null | HttpErrorResponse> {
    if (!this.streamListService.futureStreamsDto) {
      return null;
    }
    const pageData = PageDataUtil.create(this.streamListService.pastStreamsDto);
    if (!!pageData && !!nextPageData && PageDataUtil.checkNextPage(pageData, nextPageData)) {
      await this.streamListService.getPastStream(this.userId, nextPageData);
      this.changeDetector.markForCheck();
    }
    return null;
  }

  public isFuture(startTime: StringDateTime | null): boolean | null {
    return StreamDtoUtil.isFuture(startTime);
  }

  public doRedirectToStreamView(streamId: number): void {
    /*if (!!streamId) {
      this.streamService.redirectToStreamViewPage(streamId);
    }*/
  }

  // "Panel Calendar"

  /*public async doChangeSelectedDate(selectedDate: StringDate): Promise<null | HttpErrorResponse> {
    if (!selectedDate) {
      return null;
    }
    const userId = (this.userId as string);
    await this.scheduleService.setSelectedDateAndGetMiniStreams(userId, selectedDate)
      .finally(() => {
        this.changeDetector.markForCheck();
      });
    return null;
  }*/

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
    // this.doRequestNextPageFuture(PageDataUtil.create({ page: 1 }));

    this.streamListService.clearPastStream();
    this.doRequestNextPagePast(PageDataUtil.create({ page: 1 }));

    // this.doChangeActiveDate(this.scheduleService.activeDate.format(MOMENT_ISO8601_DATE));
    // this.doChangeSelectedDate(this.scheduleService.selectedDate);
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

  public doActionDelete(streamDto: StreamDto): void {
    this.alertService.hide();
    /*if (!!streamDto) {
      this.alertService.hide();
      const message = this.translateService.instant('my_streams.sure_you_want_delete_stream', { title: streamDto.title });
      this.dialogService.openConfirmation(message, '', 'buttons.no', 'buttons.yes')
        .then((result) => {
          if (!!result) {
            this.deleteDataStream(streamDto);
          }
        });
    }*/
  }

  // ** Private API **


  // "Streams"

  private async deleteDataStream(streamDto: StreamDto): Promise<void> {
    /*this.alertService.hide();
    if (!streamDto) {
      return Promise.reject();
    }
    let isRefres = false;
    this.streamService.deleteStream(streamDto.id)
      .then((response: StreamDTO | HttpErrorResponse) => {
        isRefres = true;
      })
      .catch((error: HttpErrorResponse) => {
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], 'my_streams.error_delete_stream');
        throw error;
      })
      .finally(() => {
        this.changeDetector.markForCheck();
        if (isRefres) {
          Promise.resolve().then(() => {
            this.loadFutureAndPastStreamsAndSchedule();
          });
        }
      });*/
  }
}
