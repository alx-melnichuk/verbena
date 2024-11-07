import {
  AfterContentInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, Input, OnInit, Output, ViewEncapsulation
} from '@angular/core';
import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { InitializationService } from 'src/app/common/initialization.service';
import { StringDateTime        } from 'src/app/common/string-date-time';
import { SidebarComponent      } from 'src/app/components/sidebar/sidebar.component';
import { SpinnerComponent      } from 'src/app/components/spinner/spinner.component';
import { AlertService          } from 'src/app/lib-dialog/alert.service';
import { ConfirmationData      } from 'src/app/lib-dialog/confirmation/confirmation.component';
import { DialogService         } from 'src/app/lib-dialog/dialog.service';
import { ProfileService        } from 'src/app/lib-profile/profile.service';
import { StreamService         } from 'src/app/lib-stream/stream.service';
import { StreamDto, StreamState } from 'src/app/lib-stream/stream-api.interface';
import { HttpErrorUtil         } from 'src/app/utils/http-error.util';
import { StringDateTimeUtil    } from 'src/app/utils/string-date-time.util';

import { PanelStreamActionsComponent } from '../panel-stream-actions/panel-stream-actions.component';
import { PanelStreamParamsComponent  } from '../panel-stream-params/panel-stream-params.component';
import { PanelStreamStateComponent   } from '../panel-stream-state/panel-stream-state.component';

@Component({
  selector: 'app-concept-view',
  exportAs: 'appConceptView',
  standalone: true,
  imports: [CommonModule, TranslateModule, SpinnerComponent, SidebarComponent, 
    PanelStreamStateComponent, PanelStreamParamsComponent, PanelStreamActionsComponent],
  templateUrl: './concept-view.component.html',
  styleUrls: ['./concept-view.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ConceptViewComponent implements AfterContentInit, OnInit {

//   @Input()
//   public streamId: string | null = null;
  @Input()
  public streamDto: StreamDto | null = null;
//   @Input()
//   public extendedUserDTO: ExtendedUserDTO | null = null;
//   @Input()
//   public hasSubscriptionToAuthor: boolean | null = null;
//   @Input()
//   public subscribeLoading: boolean | null = null;
//   @Input()
//   public chatMessages: ChatMessage[] = [];

  @Output()
  readonly actionSubscribe: EventEmitter<boolean> = new EventEmitter();
  @Output()
  readonly sendMessage: EventEmitter<string> = new EventEmitter();
  @Output()
  readonly removeMessage: EventEmitter<string> = new EventEmitter();
  @Output()
  readonly editMessage: EventEmitter<KeyValue<string, string>> = new EventEmitter();
  @Output()
  readonly bannedUser: EventEmitter<string> = new EventEmitter();

  public isSidebarLeftOpen: boolean = false;
  public isSidebarRightOpen: boolean = false;

  public isDarkTheme = false;
  public isMiniSidebarLeft = false;
  public isMiniSidebarRight = false;
//   public streamSetStateForbbidenDTO: StreamSetStateForbbidenDTO | null = null;
  // To disable the jumping effect of the "stream-video" panel at startup.
  public isStreamVideo = false;

  // An indication that this is the owner of the stream.
  public isStreamOwner = false;

//   // Block: "Stream Video". Contains a link to the Millicast frame.
//   public millicastViewer: MillicastViewerPrm | null = null;

  // Blocks: "Stream info" and "Stream info owner"
  public broadcastDuration: string | null = null;
  public starttimeValue: StringDateTime | null = null;
  public settimeoutId: number | null = null;

//   // Block "Chat"
//   public isEditableChat = false;
//   public isUserBanned = false;

//   private sessionDTOSub: Subscription;
//   private routerNavigationStartSub: Subscription;
//   private targetsFirebaseSub: Subscription | undefined;

  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router,
    private translateService: TranslateService,
    public initializationService: InitializationService,
    private dialogService: DialogService,
    private alertService: AlertService,
    private streamService: StreamService,
    // public socketService: SocketService,
    // public firebaseService: FirebaseService,
    // public streamInfoService: StreamInfoService,
    public profileService: ProfileService,
    // public followersService: FollowersService,
    // public streamStateService: StreamStateService,
  ) {
    // this.routerNavigationStartSub = this.router.events
    //   .pipe(filter(event => event instanceof NavigationStart))
    //   .subscribe((event: NavigationEvent) => {
    //     if (!!this.streamDto?.id) {
    //       const currentUrl = ROUTE_VIEW + '/' + this.streamDto.id;
    //       const routerEvent = (event as RouterEvent);
    //       if (!routerEvent.url.startsWith(currentUrl)) {
    //         // Disconnect from the list of viewers.
    //         this.socketService.leaveFromViewerList();
    //       }
    //     }
    //   });
    // this.isDarkTheme = (!!this.profileService.getIsDarkTheme());
    // // Subscription to change the theme of the application.
    // this.sessionDTOSub = this.profileService.sessionDTO$
    //   .subscribe((sessionDTO: SessionDTO | null) => {
    //     this.isDarkTheme = (!!this.profileService.getIsDarkTheme());
    //     this.changeDetectorRef.markForCheck();
    //   });

    // this.getFollowersAndPopular();
    this.streamDto = this.route.snapshot.data['streamDto'];
    // #if (this.streamDto != null) { // #
    // #  this.streamDto.state = StreamState.started; // #
    // #  this.streamDto.starttime = "2024-10-29T16:34:00.000Z";
    // #} // #
  }

  // To disable the jumping effect of the "stream-video" panel at startup.
  ngAfterContentInit(): void {
    this.isStreamVideo = true;
    this.changeDetectorRef.markForCheck();
  }

  ngOnInit(): void {
    const currentUserId: number = this.profileService.profileDto?.id || -1;
    // An indication that this is the owner of the stream.
    this.isStreamOwner = (this.streamDto?.userId === currentUserId);
    console.log(`#_this.isStreamOwner:${this.isStreamOwner}`); // #
    // "Stream Targets Subscribe"
    if (!!this.streamDto?.id) {
      if (this.isStreamOwner) {
        this.updateViewByStreamStatus(this.streamDto);
      } else {
        this.viewerOnlyTargetsFirebaseSubscribe(this.streamDto.id);
      }
    // // Connecting to the list of viewers.
    // this.socketService.joinViewerList(this.streamDto.id, this.profileService.getAccessToken());
    // // Subscribe to receive chat content.
    // this.firebaseService.subscribingToChatContent(this.streamDto.id);
    }
    this.broadcastDuration = '00';
  }

  // ** Public API **

  // Section: "Panel stream info"

  public doActionSubscribe(isSubscribe: boolean): void {
    this.actionSubscribe.emit(isSubscribe);
  }

  // Section: "app-panel-stream-admin"

  public getDate(starttime: StringDateTime | null | undefined): Date | null {
    return StringDateTimeUtil.toDate(starttime);
  }
  // public doActionPrepare(streamId: number): void {
  //   this.toggleStreamState(streamId, StreamState.preparing);
  // }
  // public doActionStart(streamId: number): void {
  //   this.toggleStreamState(streamId, StreamState.started);
  // }
  // public doActionStop(streamId: number): void {
    // if (!streamId || !this.streamDto) {
    //   return;
    // }
    // const message = this.translateService.instant('my_streams.sure_you_want_stop_stream', { title: this.streamDto.title });
    // const params = { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' };
    // this.dialogService.openConfirmation(message, '', params)
    //   .then((response) => {
    //     if (!!response) {
        //   this.changeDetectorRef.markForCheck();
  //         this.toggleStreamState(streamId, StreamState.stopped);
    //     }
    //   });
  // }
  // public doActionPause(streamId: number): void {
  //   this.toggleStreamState(streamId, StreamState.paused);
  // }
  public doChangeState(newState: StreamState): void {
    // console.log(`doChangeState(newState: ${newState})`) // #
    this.toggleStreamState(this.streamDto?.id || null, newState);
  }

  // Section: "Chat"

  public doSendMessage(newMessage: string): void {
    if (!!newMessage) {
      this.sendMessage.emit(newMessage);
    }
  }

  public doRemoveMessage(messageId: string): void {
    if (!!messageId) {
      this.removeMessage.emit(messageId);
    }
  }

  public doEditMessage(keyValue: KeyValue<string, string>): void {
    if (!!keyValue && !!keyValue.key) {
      this.editMessage.emit(keyValue);
    }
  }

  public doBannedUser(userId: string): void {
    if (!!userId) {
      this.bannedUser.emit(userId);
    }
  }

  // ** Private API **

  // Section: "Followers And Popular"

  private getFollowersAndPopular(): Promise<any/*ExtendedUserDTO*/[] | HttpErrorResponse> {
    return Promise.reject('Error19');
    // return this.followersService.getFollowsAndPopular()
    //   .finally(() => this.changeDetectorRef.markForCheck());
  }

  // Turn on/off the display of "Broadcast duration".
  private setBroadcastDuration(): void {
    this.broadcastDuration = '';
    const startedDate: Date | null = StringDateTimeUtil.toDate(this.starttimeValue);
    if (startedDate != null) {
      // const startedDate: moment.Moment = moment(this.starttimeValue, MOMENT_ISO8601);
      // const currentDate: moment.Moment = moment().clone();
      // const duration = moment.duration(currentDate.diff(startedDate));

      const currentDateSec = Math.floor((new Date()).getTime() / 1000);
      const startedDateSec = Math.floor(startedDate.getTime() / 1000);

      const duration = Math.floor(currentDateSec - startedDateSec); // in seconds

      // const days = Math.floor(duration.asDays());
      const days = Math.floor(duration / (24 * 60 * 60));            // (((sec / 60 :min) / 60 :hour) / 24 :day)
      const daysInHours = days * 24;
      // const hours = Math.floor(duration.asHours() - daysInHours);
      const hours = Math.floor(duration / (60 * 60) - daysInHours);  // ((sec / 60 :min) / 60 :hour)
      const hoursInMinutes = hours * 60;
      const daysInMinutes = daysInHours * 60;
      // const minutes = Math.floor(duration.asMinutes() - daysInMinutes - hoursInMinutes);
      const minutes = Math.floor(duration / 60 - daysInMinutes - hoursInMinutes);  // (sec / 60 :min)
      const minutesInSeconds = minutes * 60;
      // const seconds = Math.floor(duration.asSeconds() - daysInMinutes * 60 - hoursInMinutes * 60 - minutesInSeconds);
      const seconds = Math.floor(duration - daysInMinutes * 60 - hoursInMinutes * 60 - minutesInSeconds);

      if (days > 0) {
        this.broadcastDuration = '' + days + ' ' + (days > 1 ? 'days' : 'day') + ' ';
      }
      this.broadcastDuration += ('00' + hours).substr(-2) + ':' + ('00' + minutes).substr(-2) + ':' + ('00' + seconds).substr(-2);
      console.log(`this.broadcastDuration: ${this.broadcastDuration}`); // #
    //   this.settimeoutId = window.setTimeout(() => {
    //     this.setBroadcastDuration();
    //   }, 1000);
    } else {
    //   if (!this.settimeoutId) {
    //     window.clearTimeout(this.settimeoutId as number);
    //     this.settimeoutId = null;
    //   }
    }
    this.changeDetectorRef.markForCheck();
  }

  // Stream for Owner

  private toggleStreamState(streamId: number | null, streamState: StreamState): void {
    if (!!streamId && this.isStreamOwner) {
      this.streamService.toggleStreamState(streamId, streamState)
        .then((response: StreamDto | HttpErrorResponse) => {
          this.streamDto = (response as StreamDto);
          this.updateViewByStreamStatus(this.streamDto);
        })
        .catch((error: HttpErrorResponse) => {
          const appError = (typeof (error?.error || '') == 'object' ? error.error : {});
          const title = 'concept-view.error_update_stream';

          if (error.status == 409 && appError['code'] == 'Conflict' && appError['message'] == 'exist_is_active_stream') {
            const errParams = appError['params'] || {};
            const link = this.streamService.getLinkForVisitors(errParams['activeStream']['id'] || -1, false);
            const name = errParams['activeStream']['title'] || '';
            const confirmData: ConfirmationData = {
              messageHtml: this.translateService.instant('concept-view.exist_is_active_stream', { link, name }),
            };
            this.dialogService.openConfirmation('', title, { btnNameCancel: null, btnNameAccept: 'buttons.ok' }, { data: confirmData })
          } else {
            this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], title);
          }
          throw error;
        })
        .finally(() =>
          this.changeDetectorRef.markForCheck());
    }
  }

  // Stream for Reviewer

  private viewerOnlyTargetsFirebaseSubscribe(streamId: number): void {
    if (!streamId || this.isStreamOwner || !this.streamDto) {
      return;
    }
    // if (!!this.targetsFirebaseSub) {
    //   this.targetsFirebaseSub.unsubscribe();
    // }
    // let isInit = true;

    // this.targetsFirebaseSub = this.firebaseService.subscribingToTargets(streamId)
    //   .subscribe(async (event) => {
    //     if (!!this.streamDto) {
    //       let newStreamState: StreamState | null  = StreamStateUtil.create(event?.publicState);
    //       // console .log('event?.publicState=', event?.publicState);
    //       if (isInit) {
    //         newStreamState = (newStreamState === null ? this.streamDto.state : newStreamState);
    //         // console .log('isInit newStreamState=', newStreamState);
    //         isInit = false;
    //       }
    //       if (newStreamState === null) {
    //         const streamDTO: StreamDto = (await this.streamStateService.getStreamState(this.streamDto.id) as StreamDto);
    //         newStreamState = streamDTO.state;
    //         // console .log('getStreamState()=', streamDTO.state);
    //       }

    //       // If the event is "null", then the stream is stopped..
    //       this.streamDto.state = newStreamState; // (StreamStateUtil.create(newState) as StreamState);
    //       // Open the melikast frame to the viewer only for the "started" state.
    //       this.streamDto.publicTarget = (this.streamDto.state === StreamState.Started && !!event ? event.publicTarget : null);

    //       this.updateViewByStreamStatus(this.streamDto);
    //       this.changeDetectorRef.markForCheck();
    //     }
    //   });
  }

  // Updating data by stream

  private updateViewByStreamStatus(streamDto: StreamDto): void {
    if (!!streamDto) {
      console .log('updateView() state=', streamDto.state);
      // const src: string | null = streamDto.publicTarget;
      // this.millicastViewer = (!!src ? ({ src } as MillicastViewerPrm) : null);
      // this.isEditableChat = StreamStateUtil.isActive(streamDto.state);
      this.starttimeValue = (streamDto.state === StreamState.started ? streamDto.starttime : null);
      this.setBroadcastDuration();
    }
  }
}
