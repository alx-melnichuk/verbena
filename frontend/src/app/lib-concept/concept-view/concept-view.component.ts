import {
    AfterContentInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, inject, Input,
    Output, ViewEncapsulation
} from '@angular/core';
import { CommonModule, KeyValue } from '@angular/common';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { LocaleService } from 'src/app/common/locale.service';
import { StringDateTime } from 'src/app/common/string-date-time';
import { AvatarComponent } from 'src/app/components/avatar/avatar.component';
import { SidebarHandlerDirective } from 'src/app/components/sidebar/sidebar-handler.directive';
import { SidebarComponent } from 'src/app/components/sidebar/sidebar.component';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { ChatMessageDto, BlockedUserDto, ParamQueryPastMsg } from 'src/app/lib-chat/chat-message-api.interface';
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
export class ConceptViewComponent implements AfterContentInit {
    @Input()
    public chatBlockedUsers: string[] = []; // List of new blocked users.
    @Input()
    public chatPastMsgs: ChatMessageDto[] = []; // List of past chat messages.
    @Input()
    public chatNewMsgs: ChatMessageDto[] = []; // List of new chat messages.
    @Input()
    public chatRmvIds: number[] = []; // List of IDs of permanently deleted chat messages.
    @Input()
    public chatIsBlocked: boolean | null = null; // Indication that the user is blocked.
    @Input()
    public chatIsEditable: boolean | null = null; // Indicates that the user can send messages to the chat.
    @Input()
    public chatIsLoading: boolean | null = null; // Indicates that data is being loaded.
    @Input()
    public chatIsOwner: boolean | null | undefined; // Indicates that the user is the owner of the chat.
    @Input()
    public chatMaxLen: number | null | undefined;
    @Input()
    public chatMinLen: number | null | undefined;
    @Input()
    public chatMaxRows: number | null | undefined;
    @Input()
    public chatMinRows: number | null | undefined;
    @Input()
    public chatNickname: string | null = null;

    @Input()
    public countOfViewer: number | null | undefined;

    @Input()
    public isLoadStream = false;
    @Input()
    public isStreamOwner: boolean = false;
    @Input()
    public nickname: string | null = null;

    @Input()
    public ownerAvatar: string | null | undefined;
    @Input()
    public ownerEmail: string | null | undefined;
    @Input()
    public ownerNickname: string | null | undefined;

    @Input()
    public streamDto: StreamDto | null = null;

    @Input()
    public timerActive: boolean | null | undefined;
    @Input()
    public timerIsShow: boolean | null | undefined;
    @Input()
    public timerValue: number | null | undefined;

    @Output()
    readonly changeState: EventEmitter<StreamState> = new EventEmitter();
    @Output()
    readonly blockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly sendMessage: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly editMessage: EventEmitter<KeyValue<number, string>> = new EventEmitter();
    @Output()
    readonly cutMessage: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly rmvMessage: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly queryPastMsgs: EventEmitter<ParamQueryPastMsg> = new EventEmitter();

    public isSidebarLfOpen: boolean = false;
    public isSidebarRgOpen: boolean = true; // false;
    // To disable the jumping effect of the "stream-video" panel at startup.
    public isStreamVideo = false;

    public localeService: LocaleService = inject(LocaleService);

    // Block "Chat"

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogService: DialogService = inject(DialogService);
    private translateService: TranslateService = inject(TranslateService);

    // To disable the jumping effect of the "stream-video" panel at startup.
    ngAfterContentInit(): void {
        this.isStreamVideo = true;
        this.changeDetector.markForCheck();
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
    public doCutMessage(keyValue: KeyValue<number, string>): void {
        if (!keyValue || !keyValue.key || !keyValue.value) {
            return;
        }
        const msg = keyValue.value.slice(0, 45) + (keyValue.value.length > 45 ? '...' : '');
        const message = this.translateService.instant('concept-view.sure_you_want_delete_message', { message: msg });
        this.dialogService.openConfirmation(message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' })
            .then((res) => {
                if (!!res) {
                    this.cutMessage.emit(keyValue.key);
                }
            });

    }
    public doRmvMessage(chMsgId: number): void {
        if (!chMsgId || chMsgId < 0) {
            return;
        }
        const message = this.translateService.instant('concept-view.sure_you_want_permanently_delete_message');
        this.dialogService.openConfirmation(message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' })
            .then((res) => {
                if (!!res) {
                    this.rmvMessage.emit(chMsgId);
                }
            });
    }
    public doQueryPastMsgs(info: ParamQueryPastMsg) {
        this.queryPastMsgs.emit(info);
    }

    // ** Private API **
}
