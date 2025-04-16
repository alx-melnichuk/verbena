import {
    AfterContentInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, ElementRef, EventEmitter, Input, OnChanges, OnInit,
    Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule, KeyValue } from '@angular/common';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { InitializationService } from 'src/app/common/initialization.service';
import { LocaleService } from 'src/app/common/locale.service';
import { StringDateTime } from 'src/app/common/string-date-time';
import { AvatarComponent } from 'src/app/components/avatar/avatar.component';
import { SidebarHandlerDirective } from 'src/app/components/sidebar/sidebar-handler.directive';
import { SidebarComponent } from 'src/app/components/sidebar/sidebar.component';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { PanelChatComponent } from 'src/app/lib-chat/panel-chat/panel-chat.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileService } from 'src/app/lib-profile/profile.service';
import { StreamDto, StreamState, StreamStateUtil } from 'src/app/lib-stream/stream-api.interface';
import { StringDateTimeUtil } from 'src/app/utils/string-date-time.util';

import { PanelStreamActionsComponent } from '../panel-stream-actions/panel-stream-actions.component';
import { PanelStreamParamsComponent } from '../panel-stream-params/panel-stream-params.component';
import { PanelStreamStateComponent } from '../panel-stream-state/panel-stream-state.component';


@Component({
    selector: 'app-concept-view',
    exportAs: 'appConceptView',
    standalone: true,
    imports: [CommonModule, AvatarComponent, SpinnerComponent, SidebarComponent, TranslatePipe, PanelStreamStateComponent,
        PanelStreamParamsComponent, PanelStreamActionsComponent, PanelChatComponent, SidebarHandlerDirective],
    templateUrl: './concept-view.component.html',
    styleUrl: './concept-view.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class ConceptViewComponent implements AfterContentInit, OnChanges, OnInit {
    @Input()
    public avatar: string | null | undefined;
    @Input()
    public chatMaxRows: number | null = null;
    @Input()
    public chatMinRows: number | null = null;
    @Input()
    public isLoadStream = false;
    @Input()
    public isShowTimer: boolean = false;
    @Input()
    public isStreamOwner: boolean = false;
    @Input()
    public nickname: string | null | undefined;
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
    readonly changeState: EventEmitter<StreamState> = new EventEmitter();
    // @Output()
    // readonly actionSubscribe: EventEmitter<boolean> = new EventEmitter();
    @Output()
    readonly sendMessage: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly editMessage: EventEmitter<KeyValue<string, string>> = new EventEmitter();
    @Output()
    readonly removeMessage: EventEmitter<string> = new EventEmitter();
    // @Output()
    // readonly bannedUser: EventEmitter<string> = new EventEmitter();

    public isSidebarLfOpen: boolean = false;
    public isSidebarRgOpen: boolean = true; // false;

    // An indication that the stream is in active status. ([preparing, started, paused]) 
    public isStreamActive: boolean = false;
    //   public streamSetStateForbbidenDTO: StreamSetStateForbbidenDTO | null = null;
    // To disable the jumping effect of the "stream-video" panel at startup.
    public isStreamVideo = false;

    // Block: "Stream Video". Contains a link to the Millicast frame.
    // public millicastViewer: MillicastViewerPrm | null = null;

    // Blocks: "Stream info" and "Stream info owner"
    // public broadcastDuration: string | null = null;
    // public starttimeValue: StringDateTime | null = null;
    // public settimeoutId: number | null = null;

    // Block "Chat"
    // public isEditableChat = false;
    // public isUserBanned = false;

    // private sessionDTOSub: Subscription;
    // private routerNavigationStartSub: Subscription;
    // private targetsFirebaseSub: Subscription | undefined;
    public countOfViewer: number = -1;
    public isOverPanel: boolean = false;

    constructor(
        private changeDetector: ChangeDetectorRef,
        // private router: Router,
        public hostRef: ElementRef<HTMLElement>,
        private translateService: TranslateService,
        private dialogService: DialogService,
        public initializationService: InitializationService,
        public localeService: LocaleService,
        // private streamService: StreamService,
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

        // this.getFollowersAndPopular();

    }
    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['streamDto']) {
            this.isStreamActive = !!this.streamDto && StreamStateUtil.isActive(this.streamDto.state);
        }
    }

    // To disable the jumping effect of the "stream-video" panel at startup.
    ngAfterContentInit(): void {
        this.isStreamVideo = true;
        this.changeDetector.markForCheck();
    }

    ngOnInit(): void {
        // "Stream Targets Subscribe"
        if (!!this.streamDto?.id) {
            if (this.isStreamOwner) {
                // this.updateViewByStreamStatus(this.streamDto);
            } else {
                // this.viewerOnlyTargetsFirebaseSubscribe(this.streamDto.id);
            }
            // Connecting to the list of viewers.
            // this.socketService.joinViewerList(this.streamDto.id, this.profileService.getAccessToken());
            // Subscribe to receive chat content.
            // this.firebaseService.subscribingToChatContent(this.streamDto.id);
        }
        // this.broadcastDuration = '00';
    }

    // ** Public API **

    // ** Side Left and Right **

    public clearEvent(event: Event): void {
        event.preventDefault();
        event.stopPropagation();
    }

    // Section: "Panel stream info"
    /*
    public doActionSubscribe(isSubscribe: boolean): void {
        this.actionSubscribe.emit(isSubscribe);
    }
    */
    // Section: "app-panel-stream-admin"

    public getDate(starttime: StringDateTime | null | undefined): Date | null {
        return StringDateTimeUtil.toDate(starttime);
    }

    public doChangeState(newState: StreamState | undefined): void {
        console.log(`doChangeState(newState: ${newState})`) // #
        this.changeState.emit(newState);
        // this.toggleStreamState(this.streamDto?.id || null, newState);
    }

    // Section: "Chat"

    public doSendMessage(newMessage: string): void {
        if (!!newMessage) {
            this.sendMessage.emit(newMessage);
        }
    }
    public doEditMessage(keyValue: KeyValue<string, string>): void {
        if (!!keyValue && !!keyValue.key) {
            this.editMessage.emit(keyValue);
        }
    }
    public async doRemoveMessage(keyValue: KeyValue<string, string>): Promise<void> {
        if (!keyValue || !keyValue.key || !keyValue.value) {
            return Promise.resolve();
        }
        const msg = keyValue.value.slice(0, 45) + (keyValue.value.length > 45 ? '...' : '');
        const message = this.translateService.instant('concept-view.sure_you_want_delete_message', { message: msg });
        const res = await this.dialogService.openConfirmation(
            message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' });
        if (!!res) {
            this.removeMessage.emit(keyValue.key);
        }
    }
    /*public doRemoveMessage(messageId: string): void {
        if (!!messageId) {
            this.removeMessage.emit(messageId);
        }
    }*/
    /*public doBannedUser(userId: string): void {
        if (!!userId) {
            this.bannedUser.emit(userId);
        }
    }*/

    // ** Private API **

    // Section: "Followers And Popular"

    /*private getFollowersAndPopular(): Promise<any/ *ExtendedUserDTO* /[] | HttpErrorResponse> {
        return Promise.reject('Error19');
        return this.followersService.getFollowsAndPopular()
          .finally(() => this.changeDetector.markForCheck());
    }*/

    // Turn on/off the display of "Broadcast duration".
    /*private setBroadcastDuration(): void {
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
        this.changeDetector.markForCheck();
    }*/

    // Stream for Owner

    /*private toggleStreamState(streamId: number | null, streamState: StreamState): void {}*/

    // Stream for Reviewer

    /*private viewerOnlyTargetsFirebaseSubscribe(streamId: number): void {
        if (!streamId || this.isStreamOwner || !this.streamDto) {
            return;
        }
        if (!!this.targetsFirebaseSub) {
          this.targetsFirebaseSub.unsubscribe();
        }
        let isInit = true;

        this.targetsFirebaseSub = this.firebaseService.subscribingToTargets(streamId)
          .subscribe(async (event) => {
            if (!!this.streamDto) {
              let newStreamState: StreamState | null  = StreamStateUtil.create(event?.publicState);
              // console .log('event?.publicState=', event?.publicState);
              if (isInit) {
                newStreamState = (newStreamState === null ? this.streamDto.state : newStreamState);
                // console .log('isInit newStreamState=', newStreamState);
                isInit = false;
              }
              if (newStreamState === null) {
                const streamDTO: StreamDto = (await this.streamStateService.getStreamState(this.streamDto.id) as StreamDto);
                newStreamState = streamDTO.state;
                // console .log('getStreamState()=', streamDTO.state);
              }

              // If the event is "null", then the stream is stopped..
              this.streamDto.state = newStreamState; // (StreamStateUtil.create(newState) as StreamState);
              // Open the melikast frame to the viewer only for the "started" state.
              this.streamDto.publicTarget = (this.streamDto.state === StreamState.Started && !!event ? event.publicTarget : null);

              this.updateViewByStreamStatus(this.streamDto);
              this.changeDetector.markForCheck();
            }
        });
    }*/

    // Updating data by stream

    /*private updateViewByStreamStatus(streamDto: StreamDto): void {
        if (!!streamDto) {
            console.log('#updateView() state=', streamDto.state);
            const src: string | null = streamDto.publicTarget;
            this.millicastViewer = (!!src ? ({ src } as MillicastViewerPrm) : null);
            this.isEditableChat = StreamStateUtil.isActive(streamDto.state);
            this.starttimeValue = (streamDto.state === StreamState.started ? streamDto.starttime : null);
            this.setBroadcastDuration();
        }
    }*/
}
