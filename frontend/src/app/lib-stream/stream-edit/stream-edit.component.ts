import { ChangeDetectionStrategy, ChangeDetectorRef, Component, HostBinding, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { PanelStreamEditorComponent } from '../panel-stream-editor/panel-stream-editor.component';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';

import { ROUTE_STREAM_EDIT, ROUTE_STREAM_LIST } from 'src/app/common/routes';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

import { StreamService } from '../stream.service';
import { StreamDto, UpdateStreamFileDto } from '../stream-api.interface';

@Component({
  selector: 'app-stream-edit',
  standalone: true,
  imports: [CommonModule, SpinnerComponent, PanelStreamEditorComponent],
  templateUrl: './stream-edit.component.html',
  styleUrls: ['./stream-edit.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class StreamEditComponent {
  public isLoadDataStream = false;
  public streamDto: StreamDto;
  public errMsgs: string[] = [];
  
  private goBackToRoute: string = ROUTE_STREAM_LIST;

  @HostBinding('class.global-scroll')
  public get classGlobalScrollVal(): boolean {
    return true;
  }
  
  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router,
    private streamService: StreamService,
    private alertService: AlertService,
  ) {
    this.streamDto = this.route.snapshot.data['streamDto'];
    const previousNav = this.router.getCurrentNavigation()?.previousNavigation?.finalUrl?.toString() || ''; 
    if (!!previousNav && !previousNav.startsWith(ROUTE_STREAM_EDIT)) {
      this.goBackToRoute = previousNav;
    }
  }

  // ** Public API **

  public doCancelStream(): void {
    this.goBack();
  }

  public doUpdateStream(updateStreamFileDto: UpdateStreamFileDto): void {
    this.alertService.hide();
    if (!updateStreamFileDto) {
      return;
    }
    const isEdit = (!!updateStreamFileDto.id);
    const buffPromise: Promise<unknown>[] = [];
    this.isLoadDataStream = true;
    
    if (!!updateStreamFileDto.id) {
      buffPromise.push(this.streamService.modifyStream(updateStreamFileDto.id, updateStreamFileDto));
    } else {
      buffPromise.push(this.streamService.createStream(updateStreamFileDto));
    }
    Promise.all(buffPromise)
      .then((responses) => {
        Promise.resolve()
          .then(() => {
            this.goBack();
          });
      })
      .catch((error: HttpErrorResponse) => {
        const title = (isEdit ? 'stream_edit.error_editing_stream' : 'stream_edit.error_creating_stream');
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], title); // # del:
        this.errMsgs = HttpErrorUtil.getMsgs(error); 
        throw error;
      })
      .finally(() => {
        this.isLoadDataStream = false;
        this.changeDetectorRef.markForCheck();
      });
  }

  // ** Private API **

  private goBack(): Promise<boolean> {
    return this.router.navigateByUrl(this.goBackToRoute);
  }
}
