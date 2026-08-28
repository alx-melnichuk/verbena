import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelProfilePassword } from './panel-profile-password';

describe('PanelProfilePassword', () => {
  let component: PanelProfilePassword;
  let fixture: ComponentFixture<PanelProfilePassword>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelProfilePassword]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelProfilePassword);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
