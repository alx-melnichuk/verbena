import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ViewHugeList } from './view-huge-list';

describe('ViewHugeList', () => {
  let component: ViewHugeList;
  let fixture: ComponentFixture<ViewHugeList>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ViewHugeList]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ViewHugeList);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
