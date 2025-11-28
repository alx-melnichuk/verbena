import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBannedUsersComponent } from './panel-banned-users.component';

describe('PanelBannedUsersComponent', () => {
  let component: PanelBannedUsersComponent;
  let fixture: ComponentFixture<PanelBannedUsersComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBannedUsersComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBannedUsersComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
