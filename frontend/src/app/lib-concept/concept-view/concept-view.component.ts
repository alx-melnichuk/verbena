import {
  AfterContentInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, Input, OnChanges, OnInit, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { InitializationService } from 'src/app/common/initialization.service';
import { SidebarComponent      } from 'src/app/components/sidebar/sidebar.component';
import { DialogService         } from 'src/app/lib-dialog/dialog.service';
import { ProfileService        } from 'src/app/lib-profile/profile.service';
import { StreamDto             } from 'src/app/lib-stream/stream-api.interface';

import { PanelStreamStateComponent } from '../panel-stream-state/panel-stream-state.component';
import { PanelStreamInfoOwnerComponent } from '../panel-stream-info-owner/panel-stream-info-owner.component';

@Component({
  selector: 'app-concept-view',
  standalone: true,
  imports: [CommonModule, TranslateModule, SpinnerComponent, SidebarComponent, 
    PanelStreamStateComponent, PanelStreamInfoOwnerComponent],
  templateUrl: './concept-view.component.html',
  styleUrls: ['./concept-view.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ConceptViewComponent implements AfterContentInit, OnChanges, OnInit {

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

//   @Output()
//   readonly actionSubscribe: EventEmitter<boolean> = new EventEmitter();
//   @Output()
//   readonly sendMessage: EventEmitter<string> = new EventEmitter();
//   @Output()
//   readonly removeMessage: EventEmitter<string> = new EventEmitter();
//   @Output()
//   readonly editMessage: EventEmitter<KeyValue<string, string>> = new EventEmitter();
//   @Output()
//   readonly bannedUser: EventEmitter<string> = new EventEmitter();

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
  public starttimeValue: string | null = null;
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
    //     if (!!this.streamDTO?.id) {
    //       const currentUrl = ROUTE_VIEW + '/' + this.streamDTO.id;
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
    //     this.changeDetector.markForCheck();
    //   });

    // this.getFollowersAndPopular();
    this.streamDto = this.route.snapshot.data['streamDto'];
  }

  // To disable the jumping effect of the "stream-video" panel at startup.
  ngAfterContentInit(): void {
    this.isStreamVideo = true;
    this.changeDetectorRef.markForCheck();
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['streamDto']) {
    }
  }

  ngOnInit(): void {
    const currentUserId: number = this.profileService.profileDto?.id || -1;
    // An indication that this is the owner of the stream.
    this.isStreamOwner = (this.streamDto?.userId === currentUserId);
    console.log(`#_this.isStreamOwner:${this.isStreamOwner}`); // #
    // // "Stream Targets Subscribe"
    // if (!!this.streamDTO?.id) {
    //   if (this.isStreamOwner) {
    //     this.updateViewByStreamStatus(this.streamDTO);
    //   } else {
    //     this.viewerOnlyTargetsFirebaseSubscribe(this.streamDTO.id);
    //   }
    //   // Connecting to the list of viewers.
    //   this.socketService.joinViewerList(this.streamDTO.id, this.profileService.getAccessToken());
    //   // Subscribe to receive chat content.
    //   this.firebaseService.subscribingToChatContent(this.streamDTO.id);
    // }
    this.broadcastDuration = '00';
  }

  // ** Public API **

  // "panel-stream-state"

  public classNameViewerSize(): string {
    const isMiniLeft = this.isMiniSidebarLeft;
    return (!isMiniLeft && !this.isMiniSidebarRight ? 'size1' : (isMiniLeft && this.isMiniSidebarRight ? 'size3' : 'size2'));
  }

  // "panel-stream-info-owner"

  public doActionPrepare(streamId: number): void {}
  public doActionStart(streamId: number): void {}
  public doActionStop(streamId: number): void {}
  public doActionPause(streamId: number): void {}

  // ** Private API **

  // "Followers And Popular"

//   private getFollowersAndPopular(): Promise<ExtendedUserDTO[] | HttpErrorResponse> {
//     return this.followersService.getFollowsAndPopular()
//       .finally(() => this.changeDetector.markForCheck());
//   }

  // Turn on/off the display of "Broadcast duration".
  /*private setBroadcastDuration(): void {
    this.broadcastDuration = '';
    if (!!this.starttimeValue) {
      const startedDate: moment.Moment = moment(this.starttimeValue, MOMENT_ISO8601);
      const currentDate: moment.Moment = moment().clone();
      const duration = moment.duration(currentDate.diff(startedDate));
      const days = Math.floor(duration.asDays());
      const daysInHours = days * 24;
      const hours = Math.floor(duration.asHours() - daysInHours);
      const hoursInMinutes = hours * 60;
      const daysInMinutes = daysInHours * 60;
      const minutes = Math.floor(duration.asMinutes() - daysInMinutes - hoursInMinutes);
      const minutesInSeconds = minutes * 60;
      const seconds = Math.floor(duration.asSeconds() - daysInMinutes * 60 - hoursInMinutes * 60 - minutesInSeconds);

      if (days > 0) {
        this.broadcastDuration = '' + days + ' ' + (days > 1 ? 'days' : 'day') + ' ';
      }
      this.broadcastDuration += ('00' + hours).substr(-2) + ':' + ('00' + minutes).substr(-2) + ':' + ('00' + seconds).substr(-2);

      this.settimeoutId = window.setTimeout(() => {
        this.setBroadcastDuration();
      }, 1000);
    } else {
      if (!this.settimeoutId) {
        window.clearTimeout(this.settimeoutId as number);
        this.settimeoutId = null;
      }
    }
    this.changeDetector.markForCheck();
  }*/
}
