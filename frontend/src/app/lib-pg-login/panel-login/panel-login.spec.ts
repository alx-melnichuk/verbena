import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelLogin } from './panel-login';

describe('PanelLogin', () => {
  let component: PanelLogin;
  let fixture: ComponentFixture<PanelLogin>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelLogin]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelLogin);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
