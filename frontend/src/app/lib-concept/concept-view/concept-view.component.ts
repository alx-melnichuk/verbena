import {
    AfterContentInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, inject, Input, OnChanges, OnInit,
    Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule, KeyValue } from '@angular/common';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { LocaleService } from 'src/app/common/locale.service';
import { StringDateTime } from 'src/app/common/string-date-time';
import { AvatarComponent } from 'src/app/components/avatar/avatar.component';
import { SidebarHandlerDirective } from 'src/app/components/sidebar/sidebar-handler.directive';
import { SidebarComponent } from 'src/app/components/sidebar/sidebar.component';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { ChatMessageDto } from 'src/app/lib-chat/chat-message-api.interface';
import { PanelChatComponent } from 'src/app/lib-chat/panel-chat/panel-chat.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { StreamDto, StreamState } from 'src/app/lib-stream/stream-api.interface';
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
    public blockedUsers: string[] = [];
    @Input()
    public chatIsEditable: boolean | null = null;
    @Input()
    public chatMaxRows: number | null = null;
    @Input()
    public chatMinRows: number | null = null;
    @Input()
    public chatMsgs: ChatMessageDto[] = [];
    @Input()
    public countOfViewer: number | null | undefined;
    @Input()
    public isLoadChatMsg = false;
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

    @Output()
    readonly changeState: EventEmitter<StreamState> = new EventEmitter();
    // @Output()
    // readonly actionSubscribe: EventEmitter<boolean> = new EventEmitter();
    @Output()
    readonly blockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly sendMessage: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly editMessage: EventEmitter<KeyValue<number, string>> = new EventEmitter();
    @Output()
    readonly removeMessage: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly queryChatMsgs: EventEmitter<{ isSortDes: boolean, borderById: number }> = new EventEmitter();

    public isSidebarLfOpen: boolean = false;
    public isSidebarRgOpen: boolean = true; // false;
    public localeService: LocaleService = inject(LocaleService);

    // To disable the jumping effect of the "stream-video" panel at startup.
    public isStreamVideo = false;

    // Block: "Stream Video". Contains a link to the Millicast frame.
    // public millicastViewer: MillicastViewerPrm | null = null;

    // Blocks: "Stream info" and "Stream info owner"
    // public broadcastDuration: string | null = null;
    // public starttimeValue: StringDateTime | null = null;
    // public settimeoutId: number | null = null;

    // Block "Chat"
    // public isUserBanned = false;

    // private sessionDTOSub: Subscription;
    // private routerNavigationStartSub: Subscription;
    // private targetsFirebaseSub: Subscription | undefined;

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogService: DialogService = inject(DialogService);
    private translateService: TranslateService = inject(TranslateService);

    constructor() {
        // this.getFollowersAndPopular();
        console.log(`ConceptView()`);
    }
    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['countOfViewer']) {
            console.log(`ConceptView.OnChanges() countOfViewer: ${this.countOfViewer}`); // #
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
        }
        // this.broadcastDuration = '00';
    }

    // ** Public API **

    // ** Side Left and Right **

    public clearEvent(event: Event): void {
        event.preventDefault();
        event.stopPropagation();
    }

    // Section: "panel stream admin"

    public getDate(starttime: StringDateTime | null | undefined): Date | null {
        return StringDateTimeUtil.toDate(starttime);
    }

    public doChangeState(newState: StreamState | undefined): void {
        console.log(`doChangeState(newState: ${newState})`) // #
        this.changeState.emit(newState);
    }

    // Section: "Chat"

    public doBlockUser(user_name: string): void {
        if (!!user_name) {
            this.blockUser.emit(user_name);
        }
    }
    public doUnblockUser(user_name: string): void {
        if (!!user_name) {
            this.unblockUser.emit(user_name);
        }
    }
    public doSendMessage(newMessage: string): void {
        if (!!newMessage) {
            this.sendMessage.emit(newMessage);
        }
    }
    public doEditMessage(keyValue: KeyValue<number, string>): void {
        if (!!keyValue && !!keyValue.key) {
            this.editMessage.emit(keyValue);
        }
    }
    public async doRemoveMessage(keyValue: KeyValue<number, string>): Promise<void> {
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
    public doQueryChatMsgs(info: { isSortDes: boolean, borderById: number }) {
        this.queryChatMsgs.emit(info);
    }

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
            this.isEditableChat = StreamStateUtil. isActive(streamDto.state);
            this.starttimeValue = (streamDto.state === StreamState.started ? streamDto.starttime : null);
            this.setBroadcastDuration();
        }
    }*/

    private getChatMsg(nickname: string, len: number): ChatMessageDto[] {
        const result: ChatMessageDto[] = [];

        for (let idx = 0; idx < len; idx++) {
            let member = "Teodor_Nickols";
            let d1 = new Date((idx < (len / 2) ? -100000000 : 0) + Date.now());
            let date = d1.toISOString();
            let msg = "text_" + idx + " This function can be used to pass through a successful result while handling an error.";
            if (idx % 3 == 0) {
                member = nickname;
            } else if (idx % 2 == 0) {
                member = "Snegana_Miller";
            }
            // const date1 = date.slice(20, 24) + '_' + date.slice(11, 19) + '_' + date.slice(0, 10);
            result.push({ id: idx, date, member, msg, isEdt: false, isRmv: false });
            this.wait(1);
        }
        return result;
    }

    private wait(ms: number): void {
        const start = Date.now();
        let now = start;
        while (now - start < ms) {
            now = Date.now();
        }
    }

}
