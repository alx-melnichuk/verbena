import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBannedUsers } from './panel-banned-users';

describe('PanelBannedUsers', () => {
  let component: PanelBannedUsers;
  let fixture: ComponentFixture<PanelBannedUsers>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBannedUsers]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBannedUsers);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
