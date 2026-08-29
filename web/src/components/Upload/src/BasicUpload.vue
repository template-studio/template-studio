<template>
  <div class="w-full">
    <div class="upload">
      <div class="upload-card">
        <!--图片列表-->
        <div
          class="upload-card-item"
          :style="getCSSProperties"
          v-for="(item, index) in imgList"
          :key="`img_${index}`"
        >
          <div class="upload-card-item-info">
            <div class="img-box">
              <img :src="item" />
            </div>
            <div class="img-box-actions">
              <EyeOutlined class="mx-2 action-icon" style="font-size: 18px" @click="preview(item)" />
              <DeleteOutlined class="mx-2 action-icon" style="font-size: 18px" @click="remove(index)" />
            </div>
          </div>
        </div>

        <!--上传图片-->
        <div
          class="upload-card-item upload-card-item-select-picture"
          :style="getCSSProperties"
          v-if="imgList.length < maxNumber"
        >
          <a-upload
            class="w-auto"
            v-bind="$props"
            :showUploadList="false"
            :beforeUpload="handleBeforeUpload"
            @change="handleChange"
          >
            <div class="flex flex-col justify-center">
              <PlusOutlined style="font-size: 18px; margin: auto" />
              <span class="upload-title">上传图片</span>
            </div>
          </a-upload>
        </div>
      </div>
    </div>

    <!--上传提示-->
    <a-space>
      <a-alert :message="helpText" type="info" v-if="helpText" class="flex w-full" />
    </a-space>
  </div>

  <!--预览图片-->
  <a-modal
    v-model:open="showModal"
    title="预览"
    :footer="null"
    :style="{ width: '520px' }"
  >
    <img :src="previewUrl" style="width: 100%" />
  </a-modal>
</template>

<script lang="ts">
  import { defineComponent, toRefs, reactive, computed, watch } from 'vue';
  import { EyeOutlined, DeleteOutlined, PlusOutlined } from '@ant-design/icons-vue';
  import { message, Modal } from 'ant-design-vue';
  import { basicProps } from './props';
  import { ResultEnum } from '@/enums/httpEnum';
  import componentSetting from '@/settings/componentSetting';
  import { useGlobSetting } from '@/hooks/setting';
  import { isString } from '@/utils/is';

  const globSetting = useGlobSetting();

  export default defineComponent({
    name: 'BasicUpload',

    components: { EyeOutlined, DeleteOutlined, PlusOutlined },
    props: {
      ...basicProps,
    },
    emits: ['uploadChange', 'delete'],
    setup(props, { emit }) {
      const getCSSProperties = computed(() => {
        return {
          width: `${props.width}px`,
          height: `${props.height}px`,
        };
      });


      const state = reactive({
        showModal: false,
        previewUrl: '',
        originalImgList: [] as string[],
        imgList: [] as string[],
      });

      //赋值默认图片显示
      watch(
        () => props.value,
        () => {
          state.imgList = props.value.map((item) => {
            return getImgUrl(item);
          });
        },
        { immediate: true }
      );

      //预览
      function preview(url: string) {
        state.showModal = true;
        state.previewUrl = url;
      }

      //删除
      function remove(index: number) {
        Modal.confirm({
          title: '提示',
          content: '你确定要删除吗？',
          onOk: () => {
            state.imgList.splice(index, 1);
            state.originalImgList.splice(index, 1);
            emit('uploadChange', state.originalImgList);
            emit('delete', state.originalImgList);
          },
        });
      }

      //组装完整图片地址
      function getImgUrl(url: string): string {
        const { imgUrl } = globSetting;
        return /(^http|https:\/\/)/g.test(url) ? url : `${imgUrl}${url}`;
      }

      function checkFileType(fileType: string) {
        return componentSetting.upload.fileType.includes(fileType);
      }

      //上传之前 (Ant Design Vue beforeUpload 直接接收 File 对象)
      function handleBeforeUpload(file: File) {
        const { maxSize, accept } = props;
        const acceptRef = (isString(accept) && accept.split(',')) || [];

        // 设置最大值，则判断
        if (maxSize && file.size / 1024 / 1024 >= maxSize) {
          message.error(`上传文件最大值不能超过${maxSize}M`);
          return false;
        }

        // 设置类型,则判断
        if (acceptRef.length > 0 && !checkFileType(file.type)) {
          const fileType = componentSetting.upload.fileType;
          message.error(`只能上传文件类型为${fileType.join(',')}`);
          return false;
        }

        return true;
      }

      //上传状态变化 (Ant Design Vue @change 事件)
      function handleChange({ file }) {
        if (file.status === 'done') {
          const res = file.response;
          const infoField = componentSetting.upload.apiSetting.infoField;
          const { code } = res;
          const msg = res.msg || res.message || '上传失败';
          const result = res[infoField];
          //成功（信封统一过渡：同时认 code:0 与旧 200）
          if (code === ResultEnum.SUCCESS || code === 0) {
            let imgUrl: string = getImgUrl(result.photo);
            state.imgList.push(imgUrl);
            state.originalImgList.push(result.photo);
            emit('uploadChange', state.originalImgList);
          } else {
            message.error(msg);
          }
        } else if (file.status === 'error') {
          message.error('上传失败');
        }
      }

      return {
        ...toRefs(state),
        handleChange,
        preview,
        remove,
        handleBeforeUpload,
        getCSSProperties,
      };
    },
  });
</script>

<style lang="less">
  .upload {
    width: 100%;
    overflow: hidden;

    &-card {
      width: auto;
      height: auto;
      display: flex;
      flex-wrap: wrap;
      align-items: center;

      &-item {
        margin: 0 8px 8px 0;
        position: relative;
        padding: 8px;
        border: 1px solid #d9d9d9;
        border-radius: 2px;
        display: flex;
        justify-content: center;
        flex-direction: column;
        align-items: center;

        &:hover {
          background: 0 0;

          .upload-card-item-info::before {
            opacity: 1;
          }

          &-info::before {
            opacity: 1;
          }
        }

        &-info {
          position: relative;
          height: 100%;
          width: 100%;
          padding: 0;
          overflow: hidden;

          &:hover {
            .img-box-actions {
              opacity: 1;
            }
          }

          &::before {
            position: absolute;
            z-index: 1;
            width: 100%;
            height: 100%;
            background-color: rgba(0, 0, 0, 0.5);
            opacity: 0;
            transition: all 0.3s;
            content: ' ';
          }

          .img-box {
            position: relative;
            //padding: 8px;
            //border: 1px solid #d9d9d9;
            border-radius: 2px;
          }

          .img-box-actions {
            position: absolute;
            top: 50%;
            left: 50%;
            z-index: 10;
            white-space: nowrap;
            transform: translate(-50%, -50%);
            opacity: 0;
            transition: all 0.3s;
            display: flex;
            align-items: center;
            justify-content: space-between;

            &:hover {
              background: 0 0;
            }

            .action-icon {
              color: rgba(255, 255, 255, 0.85);

              &:hover {
                cursor: pointer;
                color: #fff;
              }
            }
          }
        }
      }

      &-item-select-picture {
        border: 1px dashed #d9d9d9;
        border-radius: 2px;
        cursor: pointer;
        background: #fafafa;
        color: #666;

        .upload-title {
          color: #666;
        }
      }
    }
  }
</style>
