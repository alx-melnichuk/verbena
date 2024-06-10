import {
  ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { TranslateModule } from '@ngx-translate/core';

import { FieldNicknameComponent } from 'src/app/components/field-nickname/field-nickname.component';
import { UserDto, UserDtoUtil } from 'src/app/lib-user/user-api.interface';
import { FieldEmailComponent } from 'src/app/components/field-email/field-email.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-panel-profile-info',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, MatButtonModule, TranslateModule, FieldNicknameComponent, FieldEmailComponent,
    FieldPasswordComponent],
  templateUrl: './panel-profile-info.component.html',
  styleUrls: ['./panel-profile-info.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelProfileInfoComponent implements OnChanges {
  @Input()
  public userInfo: UserDto | null = null;
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgList: string[] = [];
  
  public controls = {
    nickname: new FormControl(null, []),
    email: new FormControl(null, []),
    password: new FormControl(null, []),
  };
  public formGroup: FormGroup = new FormGroup(this.controls);

  private origUserDto: UserDto = UserDtoUtil.create();

  constructor() {
  }
    
  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['userInfo']) {
      this.prepareFormGroupByUserDto(this.userInfo);
    }
  }
 
  // ** Public API **
  
  public updateErrMsg(errMsgList: string[] = []): void {
    this.errMsgList = errMsgList;
  }

  public saveProfile(): void {
    console.log(`saveProfile();`);
  }

  public cancelProfile(): void {
    console.log(`cancelProfile();`);
  }

  // ** Private API **

  private prepareFormGroupByUserDto(userInfo: UserDto | null): void {
    if (!userInfo) {
      return;
    }
    this.origUserDto = { ...userInfo };
    Object.freeze(this.origUserDto);

    this.formGroup.patchValue({
      nickname: userInfo.nickname,
      email: userInfo.email,
      password: userInfo.password,
    });
  }

}
